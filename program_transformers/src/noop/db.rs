use crate::error::{ProgramTransformerError, ProgramTransformerResult};
use anchor_lang::{AnchorDeserialize, AnchorSerialize};
use digital_asset_types::dao::{
    accounts, character_history, compressed_data, compressed_data_changelog, merkle_tree,
};
use hpl_toolkit::prelude::*;
use log::{debug, error, info};
use sea_orm::{
    query::*,
    sea_query::{Expr, OnConflict},
    ActiveValue::Set,
    ColumnTrait, DbBackend, EntityTrait,
};
use serde_json::{json, Map};
use solana_sdk::pubkey::Pubkey;
use spl_account_compression::events::ApplicationDataEventV1;
use std::str::FromStr;

async fn exec_query<'c, T: ConnectionTrait + TransactionTrait>(
    txn: &T,
    query: Statement,
) -> ProgramTransformerResult<()> {
    debug!(
        "Query builed successfully, {}, values {:#?}",
        query.sql, query.values
    );
    txn.execute(query)
        .await
        .map_err(|db_err| ProgramTransformerError::StorageWriteError(db_err.to_string()))?;
    debug!("Query executed successfully");
    Ok(())
}

pub async fn save_applicationdata_event<'c, T>(
    application_data: &ApplicationDataEventV1,
    txn: &T,
) -> Result<u64, ProgramTransformerError>
where
    T: ConnectionTrait + TransactionTrait,
{
    handle_application_data(application_data, txn).await?;
    Ok(0)
}

pub async fn handle_application_data<'c, T>(
    application_data: &ApplicationDataEventV1,
    txn: &T,
) -> ProgramTransformerResult<()>
where
    T: ConnectionTrait + TransactionTrait,
{
    debug!("Inserting AppData");
    let buf = &mut &application_data.application_data[..];
    let event = CompressedDataEvent::deserialize(buf)
        .map_err(|db_err| ProgramTransformerError::CompressedDataParseError(db_err.to_string()))?;
    debug!("Application data parsed successfully");
    match event {
        CompressedDataEvent::TreeSchemaValue {
            discriminator,
            tree_id,
            schema,
            canopy_depth,
            program_id,
        } => {
            handle_tree(
                txn,
                discriminator,
                tree_id,
                schema,
                canopy_depth as i32,
                program_id,
            )
            .await?
        }
        CompressedDataEvent::Leaf {
            slot,
            tree_id,
            leaf_idx,
            seq,
            stream_type,
        } => handle_leaf(txn, tree_id, leaf_idx, stream_type, seq, slot).await?,
    }
    Ok(())
}

async fn handle_tree<'c, T: ConnectionTrait + TransactionTrait>(
    txn: &T,
    discriminator: [u8; 32],
    tree_id: [u8; 32],
    schema: Schema,
    canopy_depth: i32,
    program_id: [u8; 32],
) -> ProgramTransformerResult<()> {
    info!("Found new tree {}", bs58::encode(tree_id).into_string());
    // @TODO: Fetch and store, maxDepth, maxBufferSize, canopyDepth, etc...
    let data_schema = schema
        .try_to_vec()
        .map_err(|db_err| ProgramTransformerError::CompressedDataParseError(db_err.to_string()))?;

    debug!("Parsed tree data schema");

    let item = merkle_tree::ActiveModel {
        id: Set(tree_id.to_vec()),
        data_schema: Set(data_schema),
        discriminator: Set(discriminator.to_vec()),
        program: Set(Some(program_id.to_vec())),
        canopy_depth: Set(canopy_depth),
        ..Default::default()
    };

    let query = merkle_tree::Entity::insert(item)
        .on_conflict(
            OnConflict::columns([merkle_tree::Column::Id])
                .update_columns([merkle_tree::Column::DataSchema])
                .to_owned(),
        )
        .build(DbBackend::Postgres);
    exec_query(txn, query).await
}

async fn handle_leaf<'c, T: ConnectionTrait + TransactionTrait>(
    txn: &T,
    tree_id: [u8; 32],
    leaf_idx: u32,
    stream_type: CompressedDataEventStream,
    seq: u64,
    slot: u64,
) -> ProgramTransformerResult<()> {
    let compressed_data_id = anchor_lang::solana_program::keccak::hashv(
        &[&tree_id[..], &leaf_idx.to_le_bytes()[..]][..],
    )
    .to_bytes()
    .to_vec();
    let patch_key: Option<String>;
    let patch_data: Option<SchemaValue>;

    let leaf_idx = leaf_idx as i64;
    let seq = seq as i64;
    let slot = slot as i64;
    match stream_type {
        CompressedDataEventStream::Full { data } => {
            patch_key = None;
            patch_data = Some(data.clone());
            handle_full_leaf(txn, compressed_data_id, tree_id, leaf_idx, data, seq, slot).await?;
        }
        CompressedDataEventStream::PatchChunk { key, data } => {
            patch_key = Some(key.clone());
            patch_data = Some(data.clone());
            handle_leaf_patch(txn, compressed_data_id, key, data, slot).await?;
        }
        CompressedDataEventStream::Empty => {
            patch_key = None;
            patch_data = None;
            handle_empty_leaf(txn, compressed_data_id).await?;
        }
    }

    if let Some(data) = patch_data {
        handle_change_log(txn, tree_id, leaf_idx, patch_key, data, seq, slot).await?;
    }

    Ok(())
}

async fn handle_full_leaf<'c, T: ConnectionTrait + TransactionTrait>(
    txn: &T,
    id: Vec<u8>,
    tree_id: [u8; 32],
    leaf_idx: i64,
    mut data: SchemaValue,
    seq: i64,
    slot: i64,
) -> ProgramTransformerResult<()> {
    let tree = merkle_tree::Entity::find_by_id(tree_id.to_vec())
        .one(txn)
        .await
        .map_err(|db_err| ProgramTransformerError::StorageReadError(db_err.to_string()))?;

    debug!("Find tree query executed successfully");

    let mut schema_validated: bool = false;
    let mut program_id: Option<Pubkey> = None;
    if let Some(tree) = tree {
        debug!("Parsing tree data schema");
        let schema = Schema::deserialize(&mut &tree.data_schema[..]).map_err(|db_err| {
            ProgramTransformerError::CompressedDataParseError(db_err.to_string())
        })?;

        if tree.program.is_none() {
            return Err(ProgramTransformerError::CompressedDataParseError(format!(
                "Tree program not found"
            )));
        }
        program_id = Some(Pubkey::try_from(tree.program.unwrap()).unwrap());
        debug!("Parsed tree data schema");
        if !schema.validate(&mut data) {
            error!("Schema value validation failed");
            return Err(ProgramTransformerError::CompressedDataParseError(format!(
                "Schema value validation failed for data: {} with schema: {}",
                data.to_string(),
                schema.to_string()
            ))
            .into());
        }

        schema_validated = true;
    }

    debug!("Serializing raw data");
    let raw_data = data
        .try_to_vec()
        .map_err(|db_err| ProgramTransformerError::CompressedDataParseError(db_err.to_string()))?;
    debug!("Serialized raw data");

    let item = compressed_data::ActiveModel {
        id: Set(id.clone()),
        tree_id: Set(tree_id.to_vec()),
        leaf_idx: Set(leaf_idx),
        seq: Set(seq),
        schema_validated: Set(schema_validated),
        raw_data: Set(raw_data),
        parsed_data: Set(data.clone().into()),
        slot_updated: Set(slot),
        ..Default::default()
    };
    let query = compressed_data::Entity::insert(item)
        .on_conflict(
            OnConflict::columns([
                compressed_data::Column::TreeId,
                compressed_data::Column::LeafIdx,
            ])
            .update_columns([
                compressed_data::Column::TreeId,
                compressed_data::Column::LeafIdx,
                compressed_data::Column::Seq,
                compressed_data::Column::SchemaValidated,
                compressed_data::Column::RawData,
                compressed_data::Column::ParsedData,
                compressed_data::Column::SlotUpdated,
            ])
            .to_owned(),
        )
        .build(DbBackend::Postgres);
    exec_query(txn, query).await?;

    if let Some(program_id) = program_id {
        if program_id == Pubkey::from_str("ChRCtrG7X5kb9YncA4wuyD68DXXL8Szt3zBCCGiioBTg").unwrap() {
            if let SchemaValue::Object(character) = data {
                if let Some(kind_obj) = character.get(&"used_by".to_string()) {
                    new_character_event(
                        txn,
                        id,
                        kind_obj.clone(),
                        ("NewCharacter").to_string(),
                        slot as i64,
                        // Some(false),
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}

async fn handle_leaf_patch<'c, T: ConnectionTrait + TransactionTrait>(
    txn: &T,
    id: Vec<u8>,
    key: String,
    data: SchemaValue,
    slot: i64,
) -> ProgramTransformerResult<()> {
    let found = compressed_data::Entity::find()
        .filter(compressed_data::Column::Id.eq(id.to_owned()))
        .one(txn)
        .await
        .map_err(|db_err| ProgramTransformerError::StorageReadError(db_err.to_string()))?;

    debug!("Find old_data query executed successfully");

    if found.is_none() {
        return Err(ProgramTransformerError::StorageReadError(
            "Could not find old data in db".to_string(),
        ));
    }
    let mut db_data: compressed_data::ActiveModel = found.unwrap().into();
    debug!("Found old_data {:?}", db_data);

    let tree = merkle_tree::Entity::find_by_id(db_data.tree_id.clone().unwrap())
        .one(txn)
        .await
        .map_err(|db_err| ProgramTransformerError::StorageReadError(db_err.to_string()))?;

    debug!("Find tree query executed successfully");

    let mut program_id: Option<Pubkey> = None;
    if let Some(tree) = tree {
        program_id = Some(Pubkey::try_from(tree.program.unwrap()).unwrap());
        debug!("Parsing tree data schema");
    }

    debug!("Wrapped model into ActiveModel");

    let mut parsed_data: JsonValue = db_data.parsed_data.take().unwrap();

    if let JsonValue::Object(object) = &mut parsed_data {
        if object.contains_key(&key) {
            debug!("Patching {}: {:?}", key, data.to_string());
            if key == "used_by".to_string() {
                if let Some(program_id) = program_id {
                    debug!("program_id {:?}", program_id);
                    if program_id
                        == Pubkey::from_str("ChRCtrG7X5kb9YncA4wuyD68DXXL8Szt3zBCCGiioBTg").unwrap()
                    {
                        if let Some(used_by) = object.get("used_by") {
                            log_character_history(
                                txn,
                                id.to_owned(),
                                used_by.to_owned().into(),
                                data.to_owned(),
                                slot as i64,
                            )
                            .await?;
                        }
                    }
                }
            }

            object.insert(key, data.into());
        }
    }

    debug!("Complete Data After Patch: {}", parsed_data.to_string());
    db_data.parsed_data = Set(parsed_data);
    debug!("Data updated in object");

    db_data.slot_updated = Set(slot);

    let query: Statement = compressed_data::Entity::update(db_data)
        .filter(compressed_data::Column::Id.eq(id))
        .build(DbBackend::Postgres);
    exec_query(txn, query).await
}

async fn handle_empty_leaf<'c, T: ConnectionTrait + TransactionTrait>(
    txn: &T,
    id: Vec<u8>,
) -> ProgramTransformerResult<()> {
    let found = compressed_data::Entity::find()
        .filter(compressed_data::Column::Id.eq(id.clone()))
        .one(txn)
        .await
        .map_err(|db_err| ProgramTransformerError::StorageReadError(db_err.to_string()))?;

    debug!("Find old_data query executed successfully");

    if found.is_none() {
        return Err(ProgramTransformerError::StorageReadError(
            "Could not find old data in db".to_string(),
        ));
    }

    let db_data: compressed_data::ActiveModel = found.unwrap().into();
    debug!("Found old_data {:?}", db_data);

    let query: Statement = compressed_data::Entity::delete(db_data)
        .filter(compressed_data::Column::Id.eq(id))
        .build(DbBackend::Postgres);
    exec_query(txn, query).await
}

async fn handle_change_log<'c, T: ConnectionTrait + TransactionTrait>(
    txn: &T,
    tree_id: [u8; 32],
    leaf_idx: i64,
    key: Option<String>,
    data: SchemaValue,
    seq: i64,
    slot: i64,
) -> ProgramTransformerResult<()> {
    let change_log = compressed_data_changelog::ActiveModel {
        tree_id: Set(tree_id.to_vec()),
        leaf_idx: Set(leaf_idx),
        key: Set(key),
        data: Set(data.into()),
        seq: Set(seq),
        slot: Set(slot),
        ..Default::default()
    };

    let query = compressed_data_changelog::Entity::insert(change_log).build(DbBackend::Postgres);
    exec_query(txn, query).await
}

pub async fn log_character_history<T>(
    txn: &T,
    character_id: Vec<u8>,
    pre_used_by: SchemaValue,
    mut new_used_by: SchemaValue,
    slot: i64,
) -> Result<(), ProgramTransformerError>
where
    T: ConnectionTrait + TransactionTrait,
{
    debug!("pre_used by {:?}", pre_used_by.to_string());
    debug!("new_used by {:?}", new_used_by.to_string());
    let pre_used_by_kind = match pre_used_by {
        SchemaValue::Enum(kind, _) => kind,
        _ => unreachable!(),
    };

    let new_used_by_kind = match new_used_by.clone() {
        SchemaValue::Enum(kind, _) => kind,
        _ => unreachable!(),
    };
    debug!("pre_used_by_kind {:?}", pre_used_by_kind.to_string());
    debug!("new_used_by_kind {:?}", new_used_by_kind.to_string());

    let event = match (pre_used_by_kind.as_str(), new_used_by_kind.as_str()) {
        ("Ejected", "None") => String::from("Wrapped"),
        ("None", "Staking") => String::from("Staked"),
        ("None", "Mission") => String::from("MissionParticipation"),
        ("Staking", "None") => String::from("UnStaked"),
        ("Staking", "Staking") => String::from("ClaimedStakingReward"),
        ("Mission", "None") => String::from("RecallFromMission"),
        ("Mission", "Mission") => String::from("ClaimedMissionReward"),
        (_, "Ejected") => String::from("UnWrapped"),
        _ => "".to_string(),
    };

    if event == "".to_string() {
        debug!("Unidentified event found skipping history");
        return Ok(());
    }

    debug!("Event {:?}", event);
    debug!("Event Matched");

    if event == "RecallFromMission".to_string() {
        debug!("RecallFromMission condition matched");
        match new_used_by.clone() {
            SchemaValue::Enum(kind, params) => {
                debug!("matched new_used_by kind = {:?}", kind);
                debug!("matched new_used_by params = {:?}", params.to_string());

                if let SchemaValue::Object(object) = *params {
                    if let Some(participation_id) = object.get(&"participation_id".to_string()) {
                        // Remove the "pubkey:" prefix and convert the remaining part into a vector

                        let found = character_history::Entity::find()
                            .filter(Expr::cust_with_values(
                                "event_data->>'participation_id' = ?",
                                vec![participation_id.to_string()],
                            ))
                            .all(txn)
                            .await
                            .map_err(|db_err| {
                                ProgramTransformerError::StorageReadError(db_err.to_string())
                            })?;

                        if found.is_empty() {
                            return Err(ProgramTransformerError::StorageReadError(
                                "Could not find old character history data in db".to_string(),
                            ));
                        }
                        debug!("all event  = {:?}", found);

                        let ids = found
                            .iter()
                            .filter_map(|history| Some(history.id.clone()))
                            .collect::<Vec<i64>>();
                        debug!("all event ids = {:?}", ids);

                        update_new_used_by(
                            txn,
                            &mut new_used_by,
                            &pre_used_by_kind,
                            ids.clone(), // Ensure you have a valid id field
                            found,
                            ids.last(),
                        )
                        .await?;
                    }
                }
            }
            _ => {
                debug!("new_used_by params not condition match");
                unreachable!();
            }
        }
    }

    new_character_event(txn, character_id, new_used_by, event, slot as i64).await
}

pub async fn new_character_event<T>(
    txn: &T,
    character_id: Vec<u8>,
    event_data: SchemaValue,
    event: String,
    slot: i64,
    // fetch_history: Option<bool>,
) -> Result<(), ProgramTransformerError>
where
    T: ConnectionTrait + TransactionTrait,
{
    let found = character_history::Entity::find()
        .filter(character_history::Column::CharacterId.eq(character_id.to_owned()))
        .filter(character_history::Column::Event.eq(event.to_owned()))
        .filter(character_history::Column::SlotUpdated.eq(slot.to_owned()))
        .one(txn)
        .await
        .map_err(|db_err| ProgramTransformerError::StorageReadError(db_err.to_string()))?;

    if found.is_none() {
        let new_history = character_history::ActiveModel {
            event: Set(event), //Set(("NewCharacter").to_string()),
            event_data: Set(event_data.into()),
            character_id: Set(character_id),
            slot_updated: Set(slot),
            ..Default::default()
        };
        let query = character_history::Entity::insert(new_history)
            .on_conflict(
                OnConflict::columns([character_history::Column::Id])
                    .update_columns([character_history::Column::CharacterId])
                    .to_owned(),
            )
            .build(DbBackend::Postgres);

        exec_query(txn, query).await?;
    }
    Ok(())
}

async fn update_new_used_by<T>(
    txn: &T,
    new_used_by_value: &mut SchemaValue,
    pre_used_by_kind: &str,
    event_participant_ids: Vec<i64>,
    event_participant_data: Vec<character_history::Model>,
    last_event_id: Option<&i64>,
) -> Result<(), ProgramTransformerError>
where
    T: ConnectionTrait + TransactionTrait,
{
    match new_used_by_value {
        SchemaValue::Enum(kind, params) => {
            debug!("new_used_by params is null");
            debug!("kind = {:?}", kind);
            let mut all_rewards: Vec<Vec<JsonValue>> = Vec::new();

            for data in &event_participant_data {
                if let JsonValue::Object(object) = data.event_data.clone() {
                    if let (Some(JsonValue::Object(params)), Some(JsonValue::String(id))) =
                        (object.get("params"), object.get("id"))
                    {
                        // Remove the "pubkey:" prefix and convert the remaining part into a vector
                        let stripped_id = id.strip_prefix("pubkey:").ok_or_else(|| {
                            ProgramTransformerError::ParsingError("Invalid id format".to_string())
                        })?;
                        let id_vec: Vec<u8> =
                            bs58::decode(stripped_id).into_vec().map_err(|_| {
                                ProgramTransformerError::ParsingError(
                                    "Failed to decode id".to_string(),
                                )
                            })?;
                        let found = accounts::Entity::find_by_id(id_vec)
                            .one(txn)
                            .await
                            .map_err(|db_err| {
                                ProgramTransformerError::StorageReadError(db_err.to_string())
                            })?;

                        if found.is_none() {
                            return Err(ProgramTransformerError::StorageReadError(
                                "Could not account data in db".to_string(),
                            ));
                        }
                        let mut account: accounts::ActiveModel = found.unwrap().into();

                        let parsed_data: JsonValue =
                            account.parsed_data.take().ok_or_else(|| {
                                ProgramTransformerError::ParsingError(
                                    "Failed to take parsed_data".into(),
                                )
                            })?;

                        if let JsonValue::Object(parsed_json) = parsed_data {
                            if let (
                                Some(JsonValue::Array(parsed_data_rewards)),
                                Some(JsonValue::Array(event_data_rewards)),
                            ) = (parsed_json.get("rewards"), params.get("rewards"))
                            {
                                let mut new_rewards: Vec<JsonValue> = Vec::new();

                                for event_reward in event_data_rewards {
                                    if let Some(new_reward) =
                                        calculate_reward(event_reward, parsed_data_rewards)
                                    {
                                        new_rewards.push(new_reward.into());
                                    }
                                }

                                all_rewards.push(new_rewards.into());
                            }
                        }
                    }
                }
            }
            *new_used_by_value = SchemaValue::Enum(
                "None".to_string(),
                Box::new(SchemaValue::from(create_params(
                    pre_used_by_kind,
                    event_participant_ids.clone(),
                    all_rewards,
                    last_event_id,
                ))),
            );
        }
        _ => {
            debug!("new_used_by params not condition match");
            unreachable!();
        }
    }
    Ok(())
}

// Helper function to create a new params object

fn create_params(
    pre_used_by_kind: &str,
    event_participant_ids: Vec<i64>,
    rewards: Vec<Vec<JsonValue>>,
    last_event_id: Option<&i64>,
) -> JsonValue {
    let mut new_map = Map::new();

    // Insert `pre_used_by`
    new_map.insert(
        "pre_used_by".to_string(),
        JsonValue::String(pre_used_by_kind.to_string()),
    );

    // Convert `event_participant_ids` from Vec<i64> to JsonValue::Array and insert it
    let participant_ids_as_json_values: Vec<JsonValue> = event_participant_ids
        .into_iter()
        .map(JsonValue::from)
        .collect();

    new_map.insert(
        "event_participant_ids".to_string(),
        JsonValue::Array(participant_ids_as_json_values),
    );

    // Convert `last_event_id` to JsonValue and insert it
    let last_event_id_json_value = match last_event_id {
        Some(id) => JsonValue::from(*id), // Dereference id before passing it to JsonValue::from
        None => JsonValue::Null,
    };

    new_map.insert("last_event_id".to_string(), last_event_id_json_value);

    // Insert `rewards` directly as it's already a Vec<Vec<JsonValue>>
    new_map.insert("rewards".to_string(), JsonValue::from(rewards));

    // Return the constructed map wrapped in JsonValue::Object
    JsonValue::Object(new_map)
}

fn calculate_reward(
    event_reward: &JsonValue,
    parsed_data_rewards: &[JsonValue],
) -> Option<JsonValue> {
    if let JsonValue::Object(event_reward_obj) = event_reward {
        if let (Some(JsonValue::Number(delta)), Some(JsonValue::Number(reward_idx))) = (
            event_reward_obj.get("delta"),
            event_reward_obj.get("reward_idx"),
        ) {
            let delta = delta.as_u64()?;
            let reward_idx = reward_idx.as_u64()? as usize;

            if let Some(JsonValue::Object(min_max)) = parsed_data_rewards.get(reward_idx) {
                if let (
                    Some(JsonValue::Number(min)),
                    Some(JsonValue::Number(max)),
                    Some(JsonValue::Object(reward_type)),
                ) = (
                    min_max.get("min"),
                    min_max.get("max"),
                    min_max.get("reward_type"),
                ) {
                    let min = min.as_u64()?;
                    let max = max.as_u64()?;
                    let result = get_result_from_delta(min, max, delta);

                    return Some(json!({
                        "reward": result,
                        "reward_type": reward_type.clone()
                    }));
                }
            }
        }
    }
    None
}

pub fn get_result_from_delta(min: u64, max: u64, delta: u64) -> u64 {
    let range = max - min;
    min + ((delta * range) / 100)
}
