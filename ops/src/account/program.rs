use super::account_details::AccountDetails;
use anyhow::Result;
use clap::Parser;
use das_core::{MetricsArgs, QueueArgs, Rpc, SolanaRpcArgs};
use redis::Value as RedisValue;
use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_proto::geyser::{SubscribeUpdateAccount, SubscribeUpdateAccountInfo};
use yellowstone_grpc_proto::prost::Message;

#[derive(Debug, Parser, Clone)]
pub struct Args {
    /// Redis configuration
    #[clap(flatten)]
    pub queue: QueueArgs,

    /// Metrics configuration
    #[clap(flatten)]
    pub metrics: MetricsArgs,

    /// Solana configuration
    #[clap(flatten)]
    pub solana: SolanaRpcArgs,

    /// The batch size to use when fetching accounts
    #[arg(long, env, default_value = "1000")]
    pub batch_size: usize,

    /// The public key of the program to backfill
    #[clap(env, value_parser = parse_pubkey, use_value_delimiter = true)]
    pub programs: Vec<Pubkey>,
}

fn parse_pubkey(s: &str) -> Result<Pubkey, &'static str> {
    Pubkey::try_from(s).map_err(|_| "Failed to parse public key")
}

pub async fn run(config: Args) -> Result<()> {
    let rpc = Rpc::from_config(config.solana);

    let client = redis::Client::open(config.queue.messenger_redis_url)?;
    let mut connection = client.get_multiplexed_tokio_connection().await?;
    let mut pipe = redis::pipe();
    // let queue = QueuePool::try_from_config(config.queue).await?;

    for program in &config.programs {
        let accounts = rpc.get_program_accounts(program, None).await?;

        let accounts_chunks = accounts.chunks(config.batch_size);

        for batch in accounts_chunks {
            let results = futures::future::try_join_all(
                batch
                    .iter()
                    .map(|(pubkey, _account)| AccountDetails::fetch(&rpc, pubkey)),
            )
            .await?;

            for account_detail in results {
                let AccountDetails {
                    account: account_raw,
                    slot,
                    pubkey,
                } = account_detail;

                let account: SubscribeUpdateAccount = SubscribeUpdateAccount {
                    account: Some(SubscribeUpdateAccountInfo {
                        pubkey: pubkey.to_bytes().to_vec(),
                        lamports: account_raw.lamports,
                        owner: account_raw.owner.to_bytes().to_vec(),
                        executable: account_raw.executable,
                        rent_epoch: account_raw.rent_epoch,
                        data: account_raw.data,
                        write_version: 0,                // UNKNOWN
                        txn_signature: Some(Vec::new()), // Unknown
                    }),
                    slot,
                    is_startup: false,
                };

                pipe.xadd_maxlen(
                    &das_grpc_ingest::config::ConfigGrpcAccounts::default_stream(),
                    redis::streams::StreamMaxlen::Approx(
                        das_grpc_ingest::config::ConfigGrpcAccounts::default_stream_maxlen(),
                    ),
                    "*",
                    &[(
                        &das_grpc_ingest::config::ConfigGrpcAccounts::default_stream_data_key(),
                        account.encode_to_vec(),
                    )],
                );
            }
        }
    }

    pipe.atomic()
        .query_async::<_, RedisValue>(&mut connection)
        .await?;

    Ok(())
}
