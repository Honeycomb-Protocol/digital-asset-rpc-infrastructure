use crate::error::{ProgramTransformerError, ProgramTransformerResult};
use sea_orm::{
    query::*, sea_query::OnConflict, ActiveValue::Set, ConnectionTrait, DatabaseConnection,
    DbBackend, EntityTrait, ExecResult,
};

mod hpl_character_manager;
mod hpl_currency_manager;
mod hpl_hive_control;
mod hpl_nectar_missions;
mod hpl_nectar_staking;

pub use hpl_character_manager::handle_hpl_character_manager_account;
pub use hpl_currency_manager::handle_hpl_currency_manager_account;
pub use hpl_hive_control::handle_hpl_hive_control_account;
pub use hpl_nectar_missions::handle_hpl_nectar_missions_account;
pub use hpl_nectar_staking::handle_hpl_nectar_staking_account;

pub async fn save_account<'a, Data: hpl_toolkit::schema::ToSchema>(
    db: &'a DatabaseConnection,
    address: Vec<u8>,
    program_id: Vec<u8>,
    discriminator: Vec<u8>,
    data: &'a Data,
    slot: i64,
) -> ProgramTransformerResult<ExecResult> {
    let account = digital_asset_types::dao::accounts::ActiveModel {
        id: Set(address),
        program_id: Set(program_id),
        discriminator: Set(discriminator),
        parsed_data: Set(data.schema_value().into()),
        slot_updated: Set(slot),
        ..Default::default()
    };

    let query = digital_asset_types::dao::accounts::Entity::insert(account)
        .on_conflict(
            OnConflict::columns([digital_asset_types::dao::accounts::Column::Id])
                .update_columns([
                    digital_asset_types::dao::accounts::Column::ParsedData,
                    digital_asset_types::dao::accounts::Column::SlotUpdated,
                ])
                .to_owned(),
        )
        .build(DbBackend::Postgres);

    db.execute(query)
        .await
        .map_err(|db_err| ProgramTransformerError::StorageWriteError(db_err.to_string()))
}
