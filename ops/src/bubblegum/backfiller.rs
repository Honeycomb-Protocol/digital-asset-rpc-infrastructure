use super::tree::{TreeErrorKind, TreeGapFill, TreeGapModel, TreeResponse};
use anyhow::Result;
use base64::Engine;
use cadence_macros::{statsd_count, statsd_time};
use clap::Parser;
use das_core::{connect_db, setup_metrics, MetricsArgs, PoolArgs, QueueArgs, Rpc, SolanaRpcArgs};
use digital_asset_types::dao::cl_audits_v2;
use futures::{stream::FuturesUnordered, StreamExt};
use indicatif::HumanDuration;
use log::{debug, error, info};
use redis::aio::MultiplexedConnection;
use redis::Value as RedisValue;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, SqlxPostgresConnector,
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{
    UiInnerInstructions, UiInstruction, UiLoadedAddresses, UiTransactionReturnData,
    UiTransactionTokenBalance,
};
use std::time::Instant;
use tokio::{sync::mpsc, task::JoinHandle};
use yellowstone_grpc_proto::prelude::Message;
use yellowstone_grpc_proto::{
    geyser::{SubscribeUpdateTransaction, SubscribeUpdateTransactionInfo},
    prelude::*,
    prost::Message as OtherMessage,
};

#[derive(Debug, Parser, Clone)]
pub struct Args {
    /// Number of tree crawler workers
    #[arg(long, env, default_value = "20")]
    pub tree_crawler_count: usize,

    /// The size of the signature channel.
    #[arg(long, env, default_value = "10000")]
    pub signature_channel_size: usize,

    /// The size of the signature channel.
    #[arg(long, env, default_value = "1000")]
    pub gap_channel_size: usize,

    /// The number of transaction workers.
    #[arg(long, env, default_value = "100")]
    pub transaction_worker_count: usize,

    /// The number of gap workers.
    #[arg(long, env, default_value = "25")]
    pub gap_worker_count: usize,

    /// The list of trees to crawl. If not specified, all trees will be crawled.
    #[arg(long, env, value_parser = parse_pubkey, use_value_delimiter = true)]
    pub only_trees: Option<Vec<Pubkey>>,

    /// Database configuration
    #[clap(flatten)]
    pub database: PoolArgs,

    /// Redis configuration
    #[clap(flatten)]
    pub queue: QueueArgs,

    /// Metrics configuration
    #[clap(flatten)]
    pub metrics: MetricsArgs,

    /// Solana configuration
    #[clap(flatten)]
    pub solana: SolanaRpcArgs,

    /// The public key of the program to backfill
    #[arg(long, env, value_parser = parse_pubkey, use_value_delimiter = true)]
    pub programs: Option<Vec<Pubkey>>,
}

fn parse_pubkey(s: &str) -> Result<Pubkey, &'static str> {
    Pubkey::try_from(s).map_err(|_| "Failed to parse public key")
}

/// Runs the backfilling process for the tree crawler.
///
/// This function initializes the necessary components for the backfilling process,
/// including database connections, RPC clients, and worker managers for handling
/// transactions and gaps. It then proceeds to fetch the trees that need to be crawled
/// and manages the crawling process across multiple workers.
///
/// The function handles the following major tasks:
/// - Establishing connections to the database and initializing RPC clients.
/// - Setting up channels for communication between different parts of the system.
/// - Spawning worker managers for processing transactions and gaps.
/// - Fetching trees from the database and managing their crawling process.
/// - Reporting metrics and logging information throughout the process.
///
/// # Arguments
///
/// * `config` - A configuration object containing settings for the backfilling process,
///   including database, RPC, and worker configurations.
///
/// # Returns
///
/// This function returns a `Result` which is `Ok` if the backfilling process completes
/// successfully, or an `Err` with an appropriate error message if any part of the process
/// fails.
///
/// # Errors
///
/// This function can return errors related to database connectivity, RPC failures,
/// or issues with spawning and managing worker tasks.
pub async fn run(config: Args) -> Result<()> {
    let pool = connect_db(config.database).await?;

    let solana_rpc = Rpc::from_config(config.solana);
    let transaction_solana_rpc = solana_rpc.clone();
    let gap_solana_rpc = solana_rpc.clone();

    let client = redis::Client::open(config.queue.messenger_redis_url)?;
    let connection = client.get_multiplexed_tokio_connection().await?;

    setup_metrics(config.metrics)?;

    let (sig_sender, mut sig_receiver) = mpsc::channel::<Signature>(config.signature_channel_size);
    let gap_sig_sender = sig_sender.clone();
    let (gap_sender, mut gap_receiver) = mpsc::channel::<TreeGapFill>(config.gap_channel_size);

    let transaction_worker_count = config.transaction_worker_count;

    let transaction_worker_manager = tokio::spawn(async move {
        let mut handlers = FuturesUnordered::new();

        while let Some(signature) = sig_receiver.recv().await {
            if handlers.len() >= transaction_worker_count {
                handlers.next().await;
            }

            let solana_rpc = transaction_solana_rpc.clone();

            let handle = spawn_transaction_worker(solana_rpc, connection.clone(), signature);

            handlers.push(handle);
        }

        futures::future::join_all(handlers).await;
    });

    let gap_worker_count = config.gap_worker_count;

    let gap_worker_manager = tokio::spawn(async move {
        let mut handlers = FuturesUnordered::new();

        while let Some(gap) = gap_receiver.recv().await {
            if handlers.len() >= gap_worker_count {
                handlers.next().await;
            }

            let client = gap_solana_rpc.clone();
            let sender = gap_sig_sender.clone();

            let handle = spawn_crawl_worker(client, sender, gap);

            handlers.push(handle);
        }

        futures::future::join_all(handlers).await;
    });

    let started = Instant::now();

    debug!("{:?}", config.programs);
    let programs = config.programs.unwrap_or_default();
    let trees = if let Some(only_trees) = config.only_trees {
        debug!("Backfilling only {:?}", only_trees);
        TreeResponse::find(&solana_rpc, only_trees, &programs).await?
    } else {
        debug!("Backfilling all trees");
        TreeResponse::all(&solana_rpc, &programs).await?
    };

    let tree_count = trees.len();

    info!(
        "fetched {} trees in {}",
        tree_count,
        HumanDuration(started.elapsed())
    );

    let tree_crawler_count = config.tree_crawler_count;
    let mut crawl_handles = FuturesUnordered::new();

    for tree in trees {
        if crawl_handles.len() >= tree_crawler_count {
            crawl_handles.next().await;
        }

        let sender = gap_sender.clone();
        let pool = pool.clone();
        let conn = SqlxPostgresConnector::from_sqlx_postgres_pool(pool);

        let handle = spawn_gap_worker(conn, sender, tree);

        crawl_handles.push(handle);
    }

    futures::future::try_join_all(crawl_handles).await?;
    drop(gap_sender);
    info!("crawled all trees");

    gap_worker_manager.await?;
    drop(sig_sender);
    info!("all gaps processed");

    transaction_worker_manager.await?;
    info!("all transactions queued");

    statsd_time!("job.completed", started.elapsed());

    info!(
        "crawled {} trees in {}",
        tree_count,
        HumanDuration(started.elapsed())
    );

    Ok(())
}

fn spawn_gap_worker(
    conn: DatabaseConnection,
    sender: mpsc::Sender<TreeGapFill>,
    tree: TreeResponse,
) -> JoinHandle<Result<(), anyhow::Error>> {
    tokio::spawn(async move {
        let timing = Instant::now();

        let mut gaps = TreeGapModel::find(&conn, tree.pubkey)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        let upper_known_seq = cl_audits_v2::Entity::find()
            .filter(cl_audits_v2::Column::Tree.eq(tree.pubkey.as_ref().to_vec()))
            .order_by_desc(cl_audits_v2::Column::Seq)
            .one(&conn)
            .await?;

        let lower_known_seq = cl_audits_v2::Entity::find()
            .filter(cl_audits_v2::Column::Tree.eq(tree.pubkey.as_ref().to_vec()))
            .order_by_asc(cl_audits_v2::Column::Seq)
            .one(&conn)
            .await?;

        if let Some(upper_seq) = upper_known_seq {
            let signature = Signature::try_from(upper_seq.tx.as_ref())?;
            info!(
                "tree {} has known highest seq {} filling tree from {}",
                tree.pubkey, upper_seq.seq, signature
            );
            gaps.push(TreeGapFill::new(tree.pubkey, None, Some(signature)));
        } else if tree.seq > 0 {
            info!(
                "tree {} has no known highest seq but the actual seq is {} filling whole tree",
                tree.pubkey, tree.seq
            );
            gaps.push(TreeGapFill::new(tree.pubkey, None, None));
        }

        if let Some(lower_seq) = lower_known_seq.filter(|seq| seq.seq > 1) {
            let signature = Signature::try_from(lower_seq.tx.as_ref())?;

            info!(
                "tree {} has known lowest seq {} filling tree starting at {}",
                tree.pubkey, lower_seq.seq, signature
            );

            gaps.push(TreeGapFill::new(tree.pubkey, Some(signature), None));
        }

        let gap_count = gaps.len();

        for gap in gaps {
            if let Err(e) = sender.send(gap).await {
                statsd_count!("gap.failed", 1);
                error!("send gap: {:?}", e);
            }
        }

        info!("crawling tree {} with {} gaps", tree.pubkey, gap_count);

        statsd_count!("tree.succeeded", 1);
        statsd_time!("tree.crawled", timing.elapsed());

        Ok::<(), anyhow::Error>(())
    })
}

fn spawn_crawl_worker(
    client: Rpc,
    sender: mpsc::Sender<Signature>,
    gap: TreeGapFill,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let timing = Instant::now();

        if let Err(e) = gap.crawl(client, sender).await {
            error!("tree transaction: {:?}", e);

            statsd_count!("gap.failed", 1);
        } else {
            statsd_count!("gap.succeeded", 1);
        }

        statsd_time!("gap.queued", timing.elapsed());
    })
}

async fn queue_transaction<'a>(
    client: Rpc,
    mut connection: MultiplexedConnection,
    signature: Signature,
) -> Result<(), TreeErrorKind> {
    let transaction_raw: solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta =
        client.get_transaction(&signature).await?;
    let decoded_tx = transaction_raw.transaction.transaction.decode().unwrap();
    let signatures: Vec<Vec<u8>> = decoded_tx
        .signatures
        .iter()
        .map(|signature| <Signature as AsRef<[u8]>>::as_ref(signature).into())
        .collect();

    let transaction = SubscribeUpdateTransaction {
        transaction: Some(SubscribeUpdateTransactionInfo {
            signature: signatures[0].clone(),
            is_vote: false,
            transaction: Some(Transaction {
                signatures: signatures,
                message: {
                    let m = decoded_tx.message;
                    Some(Message {
                        header: {
                            let h = m.header();
                            Some(MessageHeader {
                                num_readonly_signed_accounts: h.num_readonly_signed_accounts as u32,
                                num_readonly_unsigned_accounts: h.num_readonly_unsigned_accounts
                                    as u32,
                                num_required_signatures: h.num_required_signatures as u32,
                            })
                        },
                        recent_blockhash: m.recent_blockhash().to_bytes().to_vec(),
                        account_keys: m
                            .static_account_keys()
                            .iter()
                            .map(|k| k.to_bytes().to_vec())
                            .collect(),
                        versioned: true,
                        address_table_lookups: m
                            .address_table_lookups()
                            .unwrap_or_default()
                            .into_iter()
                            .map(|mat| MessageAddressTableLookup {
                                readonly_indexes: mat.readonly_indexes.to_vec(),
                                writable_indexes: mat.writable_indexes.to_vec(),
                                account_key: mat.account_key.to_bytes().to_vec(),
                            })
                            .collect(),
                        instructions: m
                            .instructions()
                            .into_iter()
                            .map(|i| CompiledInstruction {
                                data: i.data.to_vec(),
                                accounts: i.accounts.to_vec(),
                                program_id_index: i.program_id_index as u32,
                            })
                            .collect(),
                    })
                },
            }),
            meta: transaction_raw.transaction.meta.map(|meta| {
                let inner_instructions = Into::<Option<Vec<UiInnerInstructions>>>::into(
                    meta.inner_instructions,
                )
                .map(|inner_instructions| {
                    inner_instructions
                        .into_iter()
                        .map(|ix| InnerInstructions {
                            index: ix.index as u32,
                            instructions: ix
                                .instructions
                                .into_iter()
                                .map(|ix| match ix {
                                    UiInstruction::Compiled(ix) => InnerInstruction {
                                        program_id_index: ix.program_id_index as u32,
                                        accounts: ix.accounts,
                                        data: bs58::decode(ix.data).into_vec().unwrap(),
                                        stack_height: ix.stack_height,
                                    },
                                    _ => todo!(),
                                })
                                .collect(),
                        })
                        .collect()
                });

                let log_messages = Into::<Option<Vec<String>>>::into(meta.log_messages);

                let pre_token_balances =
                    Into::<Option<Vec<UiTransactionTokenBalance>>>::into(meta.pre_token_balances)
                        .map(|b| {
                            b.into_iter()
                                .map(|b| TokenBalance {
                                    account_index: b.account_index as u32,
                                    mint: b.mint,
                                    ui_token_amount: Some(UiTokenAmount {
                                        ui_amount: b.ui_token_amount.ui_amount.unwrap_or_default(),
                                        decimals: b.ui_token_amount.decimals as u32,
                                        amount: b.ui_token_amount.amount,
                                        ui_amount_string: b.ui_token_amount.ui_amount_string,
                                    }),
                                    owner: Into::<Option<String>>::into(b.owner)
                                        .unwrap_or_default(),
                                    program_id: Into::<Option<String>>::into(b.program_id)
                                        .unwrap_or_default(),
                                })
                                .collect()
                        });
                let post_token_balances =
                    Into::<Option<Vec<UiTransactionTokenBalance>>>::into(meta.post_token_balances)
                        .map(|b| {
                            b.into_iter()
                                .map(|b| TokenBalance {
                                    account_index: b.account_index as u32,
                                    mint: b.mint,
                                    ui_token_amount: Some(UiTokenAmount {
                                        ui_amount: b.ui_token_amount.ui_amount.unwrap_or_default(),
                                        decimals: b.ui_token_amount.decimals as u32,
                                        amount: b.ui_token_amount.amount,
                                        ui_amount_string: b.ui_token_amount.ui_amount_string,
                                    }),
                                    owner: Into::<Option<String>>::into(b.owner)
                                        .unwrap_or_default(),
                                    program_id: Into::<Option<String>>::into(b.program_id)
                                        .unwrap_or_default(),
                                })
                                .collect()
                        });

                let (loaded_writable_addresses, loaded_readonly_addresses) =
                    Into::<Option<UiLoadedAddresses>>::into(meta.loaded_addresses)
                        .map(|x| {
                            (
                                x.writable
                                    .into_iter()
                                    .map(|a| bs58::decode(a).into_vec().unwrap())
                                    .collect(),
                                x.readonly
                                    .into_iter()
                                    .map(|a| bs58::decode(a).into_vec().unwrap())
                                    .collect(),
                            )
                        })
                        .unwrap_or((Vec::default(), Vec::default()));

                let return_data = Into::<Option<UiTransactionReturnData>>::into(meta.return_data)
                    .map(|r| ReturnData {
                        data: base64::engine::general_purpose::STANDARD
                            .decode(r.data.0)
                            .unwrap(),
                        program_id: bs58::decode(r.program_id).into_vec().unwrap(),
                    });

                TransactionStatusMeta {
                    err: None,
                    fee: meta.fee,
                    pre_balances: meta.pre_balances,
                    post_balances: meta.post_balances,
                    inner_instructions_none: inner_instructions.is_none(),
                    inner_instructions: inner_instructions.unwrap_or_default(),
                    log_messages_none: log_messages.is_none(),
                    log_messages: log_messages.unwrap_or_default(),
                    pre_token_balances: pre_token_balances.unwrap_or_default(),
                    post_token_balances: post_token_balances.unwrap_or_default(),
                    rewards: Vec::new(),
                    loaded_writable_addresses,
                    loaded_readonly_addresses,
                    return_data_none: return_data.is_none(),
                    return_data: return_data,
                    compute_units_consumed: meta.compute_units_consumed.into(),
                }
            }),
            index: 0,
        }),
        slot: transaction_raw.slot,
    };

    let mut pipe = redis::pipe();
    pipe.xadd_maxlen(
        &das_grpc_ingest::config::ConfigGrpcTransactions::default_stream(),
        redis::streams::StreamMaxlen::Approx(
            das_grpc_ingest::config::ConfigGrpcTransactions::default_stream_maxlen(),
        ),
        "*",
        &[(
            &das_grpc_ingest::config::ConfigGrpcTransactions::default_stream_data_key(),
            transaction.encode_to_vec(),
        )],
    );
    pipe.atomic()
        .query_async::<_, RedisValue>(&mut connection)
        .await
        .map_err(|_| TreeErrorKind::RedisPipe)?;

    Ok(())
}

fn spawn_transaction_worker(
    client: Rpc,
    connection: MultiplexedConnection,
    signature: Signature,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let timing = Instant::now();

        if let Err(e) = queue_transaction(client, connection, signature).await {
            error!("queue transaction: {:?}", e);

            statsd_count!("transaction.failed", 1);
        } else {
            statsd_count!("transaction.succeeded", 1);
        }

        statsd_time!("transaction.queued", timing.elapsed());
    })
}
