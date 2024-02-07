use std::{collections::HashSet, io::Cursor};

use crate::{error::IngesterError, tasks::TaskData};
use base64::engine::general_purpose;
use borsh::BorshDeserialize;
use plerkle_serialization::{CompiledInstruction, Pubkey};

use blockbuster::instruction::InstructionBundle;
use digital_asset_types::dao::accounts;
use hpl_toolkit::AccountSchemaValue;
use log::{debug, error};
use sea_orm::{
    query::*, sea_query::OnConflict, ActiveValue::Set, ConnectionTrait, DatabaseConnection,
    DbBackend, EntityTrait,
};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcSimulateTransactionConfig};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey,
    transaction::Transaction,
};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

async fn extract_account_schema_values<'a>(
    program: &'a Pubkey,
    ix: &'a CompiledInstruction<'a>,
    payer: &'a Option<pubkey::Pubkey>,
    accounts: &'a [Pubkey],
    rpc_client: &'a RpcClient,
    directory: &'a mut HashMap<pubkey::Pubkey, AccountSchemaValue>,
) {
    let program_id = pubkey::Pubkey::from(program.0);
    if payer.is_none() {
        debug!("Payer is non, aborting");
        return;
    }
    if let Some(accounts_indices) = ix.accounts() {
        let mut hash_set = HashSet::<pubkey::Pubkey>::new();
        // pubkey::Pubkey::is_on_curve(&self)
        hash_set.insert(program_id.to_owned());
        hash_set.insert(pubkey::Pubkey::default());

        let metas = accounts_indices
            .iter()
            .filter_map(|account_index| {
                let pubkey = pubkey::Pubkey::from(accounts[account_index as usize].0);
                if directory.contains_key(&pubkey)
                    || hash_set.contains(&pubkey)
                    || pubkey.is_on_curve()
                {
                    return None;
                }
                hash_set.insert(pubkey.to_owned());

                Some(AccountMeta {
                    pubkey,
                    is_signer: false,
                    is_writable: false,
                })
            })
            .collect::<Vec<AccountMeta>>();

        if metas.len() == 0 {
            return;
        }

        let simulation_ix = Instruction {
            program_id,
            accounts: metas.to_owned(),
            data: vec![215, 120, 181, 56, 249, 195, 139, 167], // discriminator for __account_schemas ix
        };
        // let message = Message::new(&[simulation_ix], Some(&metas[0].pubkey));
        // let mut tx: Transaction =
        //     Transaction::new_with_payer(&[simulation_ix], Some(&metas[0].pubkey));
        let tx: Transaction = Transaction::new_with_payer(&[simulation_ix], payer.as_ref());
        debug!("Payer: {:?}", payer.as_ref());

        // tx.message.recent_blockhash = hash;
        debug!("Simulating Tx {:?}", program_id,);

        match rpc_client
            .simulate_transaction_with_config(
                &tx,
                RpcSimulateTransactionConfig {
                    sig_verify: false,
                    commitment: Some(rpc_client.commitment()),
                    replace_recent_blockhash: true,
                    ..RpcSimulateTransactionConfig::default()
                },
            )
            .await
        {
            Ok(res) => {
                debug!("Tx Simualted success {:?}", res.value);
                if let Some(return_data) = res.value.return_data {
                    debug!("Simulate Response {}", return_data.data.0);
                    let mut wrapped_reader =
                        Cursor::new([return_data.data.0.as_bytes(), &[65; 40][..]].concat());
                    let mut decoder = base64::read::DecoderReader::new(
                        &mut wrapped_reader,
                        &general_purpose::STANDARD,
                    );

                    match Vec::<Option<AccountSchemaValue>>::deserialize_reader(&mut decoder) {
                        Ok(schema_values) => {
                            let mut i = 0;
                            schema_values.into_iter().for_each(|schema_value| {
                                if let Some(schema_value) = schema_value {
                                    let k = metas[i].pubkey;
                                    directory.insert(k, schema_value);
                                }
                                i += 1;
                            })
                        }
                        Err(error) => error!("Error deserialize_reader {:?}", error),
                    }
                } else {
                    debug!("Tx Simualted no response");
                }
            }
            Err(err) => error!("Tx simulation failed {}", err),
        }
    }
}

pub async fn etl_account_schema_values<'a, 'c>(
    ix_bundle: &'a InstructionBundle<'a>,
    accounts: &'a [Pubkey],
    payer: &'a Option<pubkey::Pubkey>,
    db: &'c DatabaseConnection,
    rpc_client: &'a RpcClient,
    _task_manager: &UnboundedSender<TaskData>,
) -> Result<(), IngesterError> {
    debug!("Checking instructions for account update");
    let mut directory = HashMap::<pubkey::Pubkey, AccountSchemaValue>::new();

    if let Some(ix) = ix_bundle.instruction {
        debug!("outer ix found fetching account schema_values");
        extract_account_schema_values(
            &ix_bundle.program,
            &ix,
            payer,
            accounts,
            rpc_client,
            &mut directory,
        )
        .await;
    }

    if let Some(inner_ixs) = &ix_bundle.inner_ix {
        debug!("inner ixs found fetching account schema_values");
        for (program_id, ix) in inner_ixs {
            extract_account_schema_values(
                &program_id,
                &ix,
                payer,
                accounts,
                rpc_client,
                &mut directory,
            )
            .await;
        }
    }

    let accounts_schemas = directory.values();

    debug!("Found {} account updates", accounts_schemas.len());

    if accounts_schemas.len() > 0 {
        debug!("Updated accounts found building query");
        let models = accounts_schemas
            .into_iter()
            .map(|account| accounts::ActiveModel {
                id: Set(account.address.to_bytes().to_vec()),
                program_id: Set(account.program_id.to_bytes().to_vec()),
                discriminator: Set(account.discriminator.to_vec()),
                parsed_data: Set(account.value.to_owned().into()),
                slot_updated: Set(ix_bundle.slot as i64),
                ..Default::default()
            })
            .collect::<Vec<accounts::ActiveModel>>();

        let query = accounts::Entity::insert_many(models)
            .on_conflict(
                OnConflict::columns([accounts::Column::Id])
                    .update_columns([
                        accounts::Column::ProgramId,
                        accounts::Column::Discriminator,
                        accounts::Column::ParsedData,
                        accounts::Column::SlotUpdated,
                    ])
                    .to_owned(),
            )
            .build(DbBackend::Postgres);

        debug!("Query builed successfully");
        db.execute(query)
            .await
            .map_err(|db_err| IngesterError::StorageWriteError(db_err.to_string()))?;
        debug!("Query executed successfully");
    }

    Ok(())
}
