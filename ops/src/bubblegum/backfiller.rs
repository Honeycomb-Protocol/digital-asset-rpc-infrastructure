use super::tree::{TreeErrorKind, TreeGapFill, TreeGapModel, TreeResponse};
use anyhow::Result;
use cadence_macros::{statsd_count, statsd_time};
use clap::Parser;
use das_core::{connect_db, setup_metrics, MetricsArgs, PoolArgs, QueueArgs, Rpc, SolanaRpcArgs};
use das_grpc_ingest::create_download_metadata_notifier;
use digital_asset_types::dao::cl_audits_v2;
use futures::{stream::FuturesUnordered, StreamExt};
use indicatif::HumanDuration;
use log::{debug, error, info};
use program_transformers::{ProgramTransformer, TransactionInfo};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, SqlxPostgresConnector};
use solana_sdk::instruction::CompiledInstruction;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::{
    InnerInstruction, InnerInstructions, UiInnerInstructions, UiInstruction,
};
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::{sync::mpsc, task::JoinHandle};

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

    /// The number of gap workers.
    #[arg(long, env, default_value = "false")]
    pub ignore_bgum: bool,

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
    // Config
    let pool = connect_db(config.database).await?;
    let solana_rpc = Rpc::from_config(config.solana);

    setup_metrics(config.metrics)?;

    let program_transformer = Arc::new(ProgramTransformer::new(
        pool.clone(),
        create_download_metadata_notifier(
            pool.clone(),
            das_grpc_ingest::config::ConfigIngesterDownloadMetadata { max_attempts: 3 },
        )?,
        true,
    ));

    // Tree Worker
    let tree_solana_rpc = solana_rpc.clone();
    let (tree_sender, mut tree_reciever) = mpsc::channel::<TreeResponse>(config.gap_channel_size);
    let tree_worker_count = 10;
    let tree_worker_manager = tokio::spawn(async move {
        let program_transformer = Arc::clone(&program_transformer);
        let mut handlers = FuturesUnordered::new();

        while let Some(tree) = tree_reciever.recv().await {
            if handlers.len() >= tree_worker_count {
                handlers.next().await;
            }
            let pool = pool.clone();
            let solana_rpc = tree_solana_rpc.clone();
            let handle = spawn_tree_worker(
                pool,
                solana_rpc,
                program_transformer.clone(),
                config.signature_channel_size,
                config.transaction_worker_count,
                tree,
            );
            handlers.push(handle);
        }

        futures::future::join_all(handlers).await
    });

    // Start crawling trees
    let started = Instant::now();

    debug!("{:?}", config.programs);
    let programs = config.programs.unwrap_or_default();
    let trees = if let Some(only_trees) = config.only_trees {
        debug!("Backfilling only {:?}", only_trees);
        TreeResponse::find(&solana_rpc, only_trees, &programs).await?
    } else {
        debug!("Backfilling all trees");
        TreeResponse::all(&solana_rpc, &programs, config.ignore_bgum).await?
    };

    let tree_count = trees.len();
    let mut tree_errored: usize = 0;

    info!(
        "fetched {} trees in {}",
        tree_count,
        HumanDuration(started.elapsed())
    );

    for tree in trees {
        let tree_id = tree.pubkey.to_string();
        match tree_sender.send(tree).await {
            Ok(_) => debug!("Sent tree {} to worker", tree_id),
            Err(e) => {
                tree_errored += 1;
                error!("While sending tree {} to worker: {:?}", tree_id, e)
            }
        }
    }
    drop(tree_sender);

    tree_worker_manager.await?;
    info!(
        "crawled {}/{} trees in {}",
        tree_count - tree_errored,
        tree_count,
        HumanDuration(started.elapsed())
    );

    Ok(())
}

fn spawn_tree_worker(
    pool: PgPool,
    client: Rpc,
    program_transformer: Arc<ProgramTransformer>,
    signature_channel_size: usize,
    transaction_worker_count: usize,
    tree: TreeResponse,
) -> JoinHandle<Result<(), anyhow::Error>> {
    tokio::spawn(async move {
        // Specific Transaction worker for this tree
        let transaction_solana_rpc = client.clone();
        let (sig_sender, mut sig_receiver) = mpsc::channel::<Signature>(signature_channel_size);
        let transaction_worker_manager = tokio::spawn(async move {
            let mut handlers = FuturesUnordered::new();

            while let Some(signature) = sig_receiver.recv().await {
                if handlers.len() >= transaction_worker_count {
                    handlers.next().await;
                }

                let solana_rpc = transaction_solana_rpc.clone();

                let handle = spawn_transaction_worker(solana_rpc, signature);

                handlers.push(handle);
            }

            futures::future::join_all(handlers).await
        });

        let gap_worker_sig_sender = sig_sender.clone();
        let gap_worker_manager = tokio::spawn(async move {
            let client = client.clone();
            let sig_sender = gap_worker_sig_sender.clone();
            let timing = Instant::now();

            let conn = SqlxPostgresConnector::from_sqlx_postgres_pool(pool);

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
                if let Err(e) = gap.crawl(client.clone(), sig_sender.clone()).await {
                    error!("tree transaction: {:?}", e);

                    statsd_count!("gap.failed", 1);
                } else {
                    statsd_count!("gap.succeeded", 1);
                }

                statsd_time!("gap.queued", timing.elapsed());
            }

            info!("crawling tree {} with {} gaps", tree.pubkey, gap_count);

            statsd_count!("tree.succeeded", 1);
            statsd_time!("tree.crawled", timing.elapsed());

            Ok::<(), anyhow::Error>(())
        });
        drop(sig_sender);
        gap_worker_manager.await??;

        let transactions = transaction_worker_manager.await?;

        for transaction in transactions {
            if let Ok(Some(transaction)) = transaction {
                program_transformer.handle_transaction(&transaction).await?;
            }
        }

        Ok(())
    })
}

async fn fetch_and_parse_transaction<'a>(
    client: Rpc,
    signature: Signature,
) -> Result<Option<TransactionInfo>, TreeErrorKind> {
    let transaction_raw: solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta =
        client.get_transaction(&signature).await?;

    if transaction_raw.transaction.meta.is_none() {
        debug!("Skipping Tx {:?} because no meta", signature);
        return Ok(None);
    }

    let meta = transaction_raw.transaction.meta.unwrap();
    debug!("tx status {:?} {:?}", meta.status, meta.err);
    if meta.status.is_err() {
        return Ok(None);
    }

    let decoded_tx = transaction_raw.transaction.transaction.decode().unwrap();

    let inner_instructions = Into::<Option<Vec<UiInnerInstructions>>>::into(
        meta.inner_instructions,
    )
    .map(|inner_instructions| {
        inner_instructions
            .into_iter()
            .map(|ix| InnerInstructions {
                index: ix.index,
                instructions: ix
                    .instructions
                    .into_iter()
                    .map(|ix| match ix {
                        UiInstruction::Compiled(ix) => InnerInstruction {
                            instruction: CompiledInstruction {
                                program_id_index: ix.program_id_index,
                                accounts: ix.accounts,
                                data: bs58::decode(ix.data).into_vec().unwrap(),
                            },
                            stack_height: ix.stack_height,
                        },
                        _ => todo!(),
                    })
                    .collect(),
            })
            .collect()
    });
    let mut account_keys = decoded_tx.message.static_account_keys().to_vec();
    if decoded_tx.message.address_table_lookups().is_some() {
        if let OptionSerializer::Some(ad) = &meta.loaded_addresses {
            for i in &ad.writable {
                account_keys.push(Pubkey::from_str(i).unwrap());
            }
            for i in &ad.readonly {
                account_keys.push(Pubkey::from_str(i).unwrap());
            }
        }
    }
    Ok(Some(TransactionInfo {
        slot: transaction_raw.slot,
        signature: decoded_tx.signatures[0],
        account_keys: account_keys,
        message_instructions: decoded_tx.message.instructions().to_vec(),
        meta_inner_instructions: inner_instructions.unwrap_or_default(),
    }))
}

fn spawn_transaction_worker(
    client: Rpc,
    signature: Signature,
) -> JoinHandle<Option<TransactionInfo>> {
    tokio::spawn(async move {
        let timing = Instant::now();

        let r = match fetch_and_parse_transaction(client, signature).await {
            Ok(transaction) => {
                statsd_count!("transaction.succeeded", 1);
                transaction
            }
            Err(e) => {
                error!("queue transaction: {:?}", e);

                statsd_count!("transaction.failed", 1);
                None
            }
        };

        statsd_time!("transaction.queued", timing.elapsed());

        r
    })
}
