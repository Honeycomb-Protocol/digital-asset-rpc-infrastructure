use crate::{error::IngesterError, tasks::TaskData};
use blockbuster::{
    instruction::{order_instructions, InstructionBundle, IxPair},
    program_handler::ProgramParser,
    programs::{
        account_compression::AccountCompressionParser, bubblegum::BubblegumParser,
        noop::NoopParser, token_account::TokenAccountParser, token_metadata::TokenMetadataParser,
        ProgramParseResult,
    },
};
use log::{debug, error, info};
use plerkle_serialization::{AccountInfo, Pubkey as FBPubkey, TransactionInfo};
use sea_orm::{DatabaseConnection, SqlxPostgresConnector};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey, pubkey::Pubkey};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet, VecDeque};
use tokio::sync::mpsc::UnboundedSender;

use crate::program_transformers::{
    account_compression::handle_account_compression_instruction,
    bubblegum::handle_bubblegum_instruction, hpl_account_handler::etl_account_schema_values,
    noop::handle_noop_instruction, token::handle_token_program_account,
    token_metadata::handle_token_metadata_account,
};

mod account_compression;
mod bubblegum;
mod hpl_account_handler;
mod noop;
mod token;
mod token_metadata;

pub struct IndexablePrograms(pub Vec<Pubkey>);
impl IndexablePrograms {
    pub fn new() -> Self {
        let mut this = Self(vec![]);
        this.populate_programs();
        this
    }

    pub fn keys(&self) -> &Vec<Pubkey> {
        &self.0
    }

    pub fn populate_programs(&mut self) {
        self.0
            .push(pubkey!("EtXbhgWbWEWamyoNbSRyN5qFXjFbw8utJDHvBkQKXLSL")); // Test HiveControl
        self.0
            .push(pubkey!("HivezrprVqHR6APKKQkkLHmUG8waZorXexEBRZWh5LRm")); // HiveControl
        self.0
            .push(pubkey!("ChRCtrG7X5kb9YncA4wuyD68DXXL8Szt3zBCCGiioBTg")); // CharacterManager
        self.0
            .push(pubkey!("CrncyaGmZfWvpxRcpHEkSrqeeyQsdn4MAedo9KuARAc4")); // Currency
        self.0
            .push(pubkey!("Pay9ZxrVRXjt9Da8qpwqq4yBRvvrfx3STWnKK4FstPr")); // Payment
        self.0
            .push(pubkey!("MiNESdRXUSmWY7NkAKdW9nMkjJZCaucguY3MDvkSmr6")); // Staking
        self.0
            .push(pubkey!("8fTwUdyGfDAcmdu8X4uWb2vBHzseKGXnxZUpZ2D94iit")); // Test GuildKit
        self.0
            .push(pubkey!("6ARwjKsMY2P3eLEWhdoU5czNezw3Qg6jEfbmLTVQqrPQ")); // Test ResourceManager
    }
}

pub struct ProgramTransformer {
    storage: DatabaseConnection,
    rpc_client: Option<RpcClient>,
    task_sender: UnboundedSender<TaskData>,
    matchers: HashMap<Pubkey, Box<dyn ProgramParser>>,
    indexable_programs: IndexablePrograms,
    key_set: HashSet<Pubkey>,
    cl_audits: bool,
}

impl ProgramTransformer {
    pub fn new(pool: PgPool, task_sender: UnboundedSender<TaskData>, cl_audits: bool) -> Self {
        let mut matchers: HashMap<Pubkey, Box<dyn ProgramParser>> = HashMap::with_capacity(1);
        let bgum: BubblegumParser = BubblegumParser {};
        let account_compression = AccountCompressionParser {};
        let token_metadata = TokenMetadataParser {};
        let token = TokenAccountParser {};
        let noop = NoopParser {};
        matchers.insert(bgum.key(), Box::new(bgum));
        matchers.insert(account_compression.key(), Box::new(account_compression));
        matchers.insert(token_metadata.key(), Box::new(token_metadata));
        matchers.insert(token.key(), Box::new(token));
        matchers.insert(noop.key(), Box::new(noop));

        let mut indexable_programs = IndexablePrograms::new();

        let mut hs = matchers.iter().fold(HashSet::new(), |mut acc, (k, _)| {
            acc.insert(*k);
            acc
        });
        indexable_programs.keys().iter().for_each(|key| {
            hs.insert(key.clone());
        });

        let pool: PgPool = pool;
        ProgramTransformer {
            storage: SqlxPostgresConnector::from_sqlx_postgres_pool(pool),
            rpc_client: None,
            task_sender,
            matchers,
            indexable_programs,
            key_set: hs,
            cl_audits,
        }
    }

    pub fn new_with_rpc_client(
        pool: PgPool,
        rpc_client: RpcClient,
        task_sender: UnboundedSender<TaskData>,
        cl_audits: bool,
    ) -> Self {
        let mut this = Self::new(pool, task_sender, cl_audits);
        this.rpc_client = Some(rpc_client);
        this
    }
    pub fn break_transaction<'i>(
        &self,
        tx: &'i TransactionInfo<'i>,
    ) -> VecDeque<(IxPair<'i>, Option<Vec<IxPair<'i>>>)> {
        let ref_set: HashSet<&[u8]> = self.key_set.iter().map(|k| k.as_ref()).collect();
        order_instructions(ref_set, tx)
    }

    #[allow(clippy::borrowed_box)]
    pub fn match_program(&self, key: &FBPubkey) -> Option<&Box<dyn ProgramParser>> {
        match Pubkey::try_from(key.0.as_slice()) {
            Ok(pubkey) => self.matchers.get(&pubkey),
            Err(_error) => {
                log::warn!("failed to parse key: {key:?}");
                None
            }
        }
    }

    pub async fn handle_transaction<'a>(
        &self,
        tx: &'a TransactionInfo<'a>,
    ) -> Result<(), IngesterError> {
        let sig: Option<&str> = tx.signature();
        info!("Handling Transaction: {:?}", sig);
        let instructions = self.break_transaction(tx);
        let accounts = tx.account_keys().unwrap_or_default();
        let slot = tx.slot();
        let txn_id = tx.signature().unwrap_or("");
        let mut keys: Vec<FBPubkey> = Vec::with_capacity(accounts.len());
        for k in accounts.into_iter() {
            keys.push(*k);
        }
        let payer = keys.get(0).map(|fk| Pubkey::from(fk.0));

        let mut not_impl = 0;
        let ixlen = instructions.len();
        debug!("Instructions: {}", ixlen);

        for (outer_ix, inner_ix) in instructions {
            let (program, instruction) = outer_ix;
            let ix_accounts = instruction.accounts().unwrap().iter().collect::<Vec<_>>();
            let ix_account_len = ix_accounts.len();
            let max = ix_accounts.iter().max().copied().unwrap_or(0) as usize;
            if keys.len() < max {
                return Err(IngesterError::DeserializationError(
                    "Missing Accounts in Serialized Ixn/Txn".to_string(),
                ));
            }
            let ix_accounts =
                ix_accounts
                    .iter()
                    .fold(Vec::with_capacity(ix_account_len), |mut acc, a| {
                        if let Some(key) = keys.get(*a as usize) {
                            acc.push(*key);
                        }
                        acc
                    });
            let ix = InstructionBundle {
                txn_id,
                program,
                instruction: Some(instruction),
                inner_ix,
                keys: ix_accounts.as_slice(),
                slot,
            };

            if let Some(program) = self.match_program(&ix.program) {
                debug!("Found a ix for program: {:?}", program.key());
                let result = program.handle_instruction(&ix)?;
                let concrete = result.result_type();
                match concrete {
                    ProgramParseResult::Bubblegum(parsing_result) => {
                        handle_bubblegum_instruction(
                            parsing_result,
                            &ix,
                            &self.storage,
                            &self.task_sender,
                            self.cl_audits,
                        )
                        .await
                        .map_err(|err| {
                            error!(
                                "Failed to handle bubblegum instruction for txn {:?}: {:?}",
                                sig, err
                            );
                            err
                        })?;
                    }
                    ProgramParseResult::AccountCompression(parsing_result) => {
                        handle_account_compression_instruction(
                            parsing_result,
                            &ix,
                            &self.storage,
                            &self.task_sender,
                            self.cl_audits,
                        )
                        .await
                        .map_err(|err| {
                            error!(
                                "Failed to handle account compression instruction for txn {:?}: {:?}",
                                sig, err
                            );
                            return err;
                        })?;
                    }
                    ProgramParseResult::Noop(parsing_result) => {
                        debug!("Handling NOOP Instruction");
                        match handle_noop_instruction(
                            parsing_result,
                            &ix,
                            &self.storage,
                            &self.task_sender,
                            self.cl_audits,
                        )
                        .await
                        {
                            Ok(_) => {}
                            Err(err) => {
                                error!(
                                    "Failed to handle noop instruction for txn {:?}: {:?}",
                                    sig, err
                                );
                            }
                        }
                    }
                    _ => {
                        not_impl += 1;
                        debug!("Could not handle this ix")
                    }
                };
            }
            if let Some(rpc_client) = &self.rpc_client {
                let mut remaining_tries = 200u64;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
                    let current_slot = rpc_client
                        .get_slot_with_commitment(rpc_client.commitment())
                        .await;
                    let current_slot = current_slot.unwrap_or(0);

                    info!(
                        "Checking confirmation for tx: {:?}, current_slot: {}, tx_slot: {}",
                        sig,
                        current_slot,
                        tx.slot()
                    );

                    if current_slot >= tx.slot() {
                        info!(
                            "Fetching account values for tx: {:?}, remaining tries: {}",
                            sig, remaining_tries
                        );
                        etl_account_schema_values(
                            self.indexable_programs.keys(),
                            keys.as_slice(),
                            tx.slot(),
                            &payer,
                            &self.storage,
                            rpc_client,
                            &self.task_sender,
                        )
                        .await
                        .map_err(|err| {
                            error!(
                                "Failed to handle bubblegum instruction for txn {:?}: {:?}",
                                sig, err
                            );
                            err
                        })?;
                        break;
                    }

                    if remaining_tries == 0 {
                        info!(
                            "Coudn't confirm, tx: {:?}, current_slot: {}, tx_slot: {}",
                            sig,
                            current_slot,
                            tx.slot()
                        );
                        break;
                    }
                    remaining_tries -= 1;
                }
            }
        }

        if not_impl == ixlen {
            debug!("Not imple");
            return Err(IngesterError::NotImplemented);
        }
        Ok(())
    }

    pub async fn handle_account_update<'b>(
        &self,
        acct: AccountInfo<'b>,
    ) -> Result<(), IngesterError> {
        let owner = acct.owner().unwrap();
        if let Some(program) = self.match_program(owner) {
            let result = program.handle_account(&acct)?;
            let concrete = result.result_type();
            match concrete {
                ProgramParseResult::TokenMetadata(parsing_result) => {
                    handle_token_metadata_account(
                        &acct,
                        parsing_result,
                        &self.storage,
                        &self.task_sender,
                    )
                    .await
                }
                ProgramParseResult::TokenProgramAccount(parsing_result) => {
                    handle_token_program_account(
                        &acct,
                        parsing_result,
                        &self.storage,
                        &self.task_sender,
                    )
                    .await
                }
                _ => Err(IngesterError::NotImplemented),
            }?;
        }
        Ok(())
    }
}
