use blockbuster::programs::{
    hpl_character_manager::{self, hpl_character_manager_id, AssemblerConfig, CharacterModel},
    hpl_hive_control::{hpl_hive_control, Global, Project},
};
use digital_asset_types::dao::accounts;
use sea_orm::{EntityTrait, SqlxPostgresConnector};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use sqlx::PgPool;
use std::str::FromStr;

pub fn extract_trees_from_parsed_data(
    parsed_data: Value,
    key: &str,
    merkle_trees: &mut Vec<Pubkey>,
) {
    if let Some(obj) = parsed_data.as_object() {
        if let Some(Some(controlled_merkle_trees)) = obj.get(key).map(|a| a.as_object()) {
            if let Some(Some(trees)) = controlled_merkle_trees
                .get("merkle_trees")
                .map(|x| x.as_array())
            {
                for tree in trees {
                    if let Some(pubkey) = tree.as_str() {
                        merkle_trees
                            .push(Pubkey::from_str(pubkey.replace("pubkey:", "").as_str()).unwrap())
                    }
                }
            }
        }
    }
}

pub async fn fetch_hc_trees(pool: PgPool) -> Result<Vec<Pubkey>, anyhow::Error> {
    let mut merkle_trees = Vec::<Pubkey>::new();

    let conn = SqlxPostgresConnector::from_sqlx_postgres_pool(pool);

    let accounts = accounts::Entity::find().all(&conn).await?;
    for account in accounts {
        let program_id = Pubkey::try_from(account.program_id).unwrap();
        let disc: [u8; 8] = account.discriminator.try_into().unwrap();

        if program_id == hpl_hive_control() {
            match disc {
                Global::DISCRIMINATOR => extract_trees_from_parsed_data(
                    account.parsed_data,
                    "user_trees",
                    &mut merkle_trees,
                ),
                Project::DISCRIMINATOR => extract_trees_from_parsed_data(
                    account.parsed_data,
                    "profile_trees",
                    &mut merkle_trees,
                ),
                _ => {}
            }
        } else if program_id == hpl_character_manager_id() {
            match disc {
                CharacterModel::DISCRIMINATOR => extract_trees_from_parsed_data(
                    account.parsed_data,
                    "merkle_trees",
                    &mut merkle_trees,
                ),
                AssemblerConfig::DISCRIMINATOR => extract_trees_from_parsed_data(
                    account.parsed_data,
                    "merkle_trees",
                    &mut merkle_trees,
                ),
                _ => {}
            }
        }
    }

    Ok(merkle_trees)
}
