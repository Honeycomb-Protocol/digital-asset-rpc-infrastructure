use blockbuster::program_handler::ProgramParser;
use blockbuster::programs::token_account::TokenAccountParser;
use blockbuster::programs::token_metadata::TokenMetadataParser;
use blockbuster::programs::ProgramParseResult;
use blockbuster::token_metadata::solana_program::pubkey::Pubkey;
use figment::{providers::Env, value::Value, Figment};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use plerkle_serialization::root_as_account_info;
use reqwest;
use solana_snapshot_etl::append_vec::{AppendVec, StoredAccountMeta};
use solana_snapshot_etl::archived::ArchiveSnapshotExtractor;
use solana_snapshot_etl::SnapshotExtractor;
use sqlx::{self, postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use std::sync::Arc;

struct Worker<'a> {
    db: &'a Pool<Postgres>,
    progress: Arc<Progress>,
}

impl<'a> AppendVecConsumer for Worker<'a> {
    fn on_append_vec(&mut self, append_vec: AppendVec) -> GenericResult<()> {
        for acc in append_vec_iter(Rc::new(append_vec)) {
            let meta: &StoredAccountMeta = &acc.access().unwrap();
            self.progress.accounts_counter.inc();

            let conn = self.db.acquire().await?;
            let c =
                plerkle_serialization::solana_geyser_plugin_interface_shims::ReplicaAccountInfoV2 {
                    pubkey: meta.account_meta.pubkey,
                    lamports: meta.account_meta.lamports,
                    owner: meta.account_meta.owner,
                    executable: meta.account_meta.executable,
                    rent_epoch: 0,
                    data: meta.data,
                    write_version: 0,
                    txn_signature: None,
                };
            let mut builder = FlatBufferBuilder::new();
            let sera =
                plerkle_serialization::serializer::serialize_account(&mut builder, &c, 0, false);
            let buf = sera.finished_data();
            let obj = root_as_account_info(buf).unwrap();
            let token_metadata = TokenMetadataParser {};
            let token = TokenAccountParser {};
            let key = Pubkey::new(c.pubkey);
            if token.key_match(&key) {
                if let ProgramParseResult::TokenProgramAccount(pr) = token.handle_account(&acct)? {
                    match 
                    
                }
            }
            if token_metadata.key_match(&key) {
                if let ProgramParseResult::TokenMetadata(pr) =
                    token_metadata.handle_account(&acct)?
                {}
            }
        }
        Ok(())
    }
}

fn main() {
    //let url = env::var("SNAPSHOT_URL").unwrap();
    let url = config
        .database_config
        .get(DATABASE_URL_KEY)
        .and_then(|u| u.clone().into_string())
        .ok_or(IngesterError::ConfigurationError {
            msg: format!("Database connection string missing: {}", DATABASE_URL_KEY),
        })
        .unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(config.max_postgres_connections.unwrap_or(100))
        .connect(&url)
        .await
        .unwrap();

    let resp = reqwest::blocking::get("https://api.metaplex.solana.com/snapshot.tar.bz2").unwrap();
    let loader = ArchiveSnapshotExtractor::from_reader(resp).unwrap();

    let mut worker = Worker {
        db: &pool,
        progress: Arc::new(Progress::new()),
    };

    let ai = loader.iter();

    for append_vec in ai {
        worker.on_append_vec(append_vec?)?;
    }
}
