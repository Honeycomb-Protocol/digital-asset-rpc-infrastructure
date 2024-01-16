use crate::error::IngesterError;
use anchor_lang::prelude::borsh::{BorshDeserialize, BorshSerialize};
use digital_asset_types::dao::{compressed_data, merkle_tree};
use hpl_compression::{CompressedDataEvent, CompressedDataEventStream, Schema, SchemaValue};
use log::{debug, info};
use sea_orm::{
    query::*, sea_query::OnConflict, ActiveValue::Set, ColumnTrait, DbBackend, EntityTrait,
};
use spl_account_compression::events::ApplicationDataEventV1;

pub async fn save_applicationdata_event<'c, T>(
    application_data: &ApplicationDataEventV1,
    txn: &T,
) -> Result<u64, IngesterError>
where
    T: ConnectionTrait + TransactionTrait,
{
    insert_application_data(application_data, txn).await?;
    Ok(0)
}

pub async fn insert_application_data<'c, T>(
    application_data: &ApplicationDataEventV1,
    txn: &T,
) -> Result<(), IngesterError>
where
    T: ConnectionTrait + TransactionTrait,
{
    debug!("Inserting AppData");
    let buf = &mut &application_data.application_data[..];
    let event = CompressedDataEvent::deserialize(buf)
        .map_err(|db_err| IngesterError::CompressedDataParseError(db_err.to_string()))?;
    debug!("Application data parsed successfully");
    match event {
        CompressedDataEvent::TreeSchemaValue { tree_id, schema } => {
            info!("Found new tree {}", bs58::encode(tree_id).into_string());

            let data_schema = schema
                .try_to_vec()
                .map_err(|db_err| IngesterError::CompressedDataParseError(db_err.to_string()))?;

            debug!("Parsed tree data schema");

            let item = merkle_tree::ActiveModel {
                id: Set(tree_id.to_vec()),
                data_schema: Set(data_schema),
                ..Default::default()
            };

            let query = merkle_tree::Entity::insert(item)
                .on_conflict(
                    OnConflict::columns([merkle_tree::Column::Id])
                        .update_columns([merkle_tree::Column::DataSchema])
                        .to_owned(),
                )
                .build(DbBackend::Postgres);
            debug!("Query builed successfully");
            txn.execute(query)
                .await
                .map_err(|db_err| IngesterError::StorageWriteError(db_err.to_string()))?;
            debug!("Query executed successfully");
        }
        CompressedDataEvent::Leaf {
            slot,
            tree_id,
            leaf_idx,
            seq,
            stream_type,
        } => match stream_type {
            CompressedDataEventStream::Full { mut data } => {
                info!(
                    "Found new leaf for {} at index {}",
                    bs58::encode(tree_id).into_string(),
                    leaf_idx
                );
                let tree = merkle_tree::Entity::find_by_id(tree_id.to_vec())
                    .one(txn)
                    .await
                    .map_err(|db_err| IngesterError::StorageReadError(db_err.to_string()))?;

                debug!("Find tree query executed successfully");

                let mut schema_validated: bool = false;
                if let Some(tree) = tree {
                    debug!("Parsing tree data schema");
                    let schema =
                        Schema::deserialize(&mut &tree.data_schema[..]).map_err(|db_err| {
                            IngesterError::CompressedDataParseError(db_err.to_string())
                        })?;

                    debug!("Parsed tree data schema");
                    if !schema.validate(&mut data) {
                        return Err(IngesterError::CompressedDataParseError(format!(
                            "Schema value validation failed for data: {} with schema: {}",
                            data.to_string(),
                            schema.to_string()
                        ))
                        .into());
                    }

                    schema_validated = true;
                }

                debug!("Serializing raw data");
                let raw_data = data.try_to_vec().map_err(|db_err| {
                    IngesterError::CompressedDataParseError(db_err.to_string())
                })?;
                debug!("Serialized raw data");

                let item = compressed_data::ActiveModel {
                    tree_id: Set(tree_id.to_vec()),
                    leaf_idx: Set(leaf_idx as i64),
                    seq: Set(seq as i64),
                    schema_validated: Set(schema_validated),
                    raw_data: Set(raw_data),
                    parsed_data: Set(data.into()),
                    slot_updated: Set(slot as i64),
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

                debug!("Query builed successfully");
                txn.execute(query)
                    .await
                    .map_err(|db_err| IngesterError::StorageWriteError(db_err.to_string()))?;
                debug!("Query executed successfully");
            }
            CompressedDataEventStream::PatchChunk { key, data } => {
                info!(
                    "Patch leaf for {} at index {}",
                    bs58::encode(tree_id).into_string(),
                    leaf_idx
                );
                let found = compressed_data::Entity::find()
                    .filter(compressed_data::Column::TreeId.eq(tree_id.to_vec()))
                    .filter(compressed_data::Column::LeafIdx.eq(leaf_idx as i64))
                    .one(txn)
                    .await
                    .map_err(|db_err| IngesterError::StorageReadError(db_err.to_string()))?;

                debug!("Find old_data query executed successfully");

                if found.is_none() {
                    return Err(IngesterError::StorageReadError(
                        "Could not find old data in db".to_string(),
                    ));
                }

                debug!("Found old_data");

                let mut db_data: compressed_data::ActiveModel = found.unwrap().into();

                debug!("Wrapped model into ActiveModel");

                let mut schema_value: SchemaValue = db_data.parsed_data.take().unwrap().into();

                debug!("JsonValue to SchemaValue successful");

                if let SchemaValue::Object(object) = &mut schema_value {
                    debug!("SchemaValue is object");
                    if let Some(SchemaValue::Object(v1_map)) = object.get_mut("V1") {
                        debug!("SchemaValue has V1");
                        let json: serde_json::Value = data.clone().into();
                        let k = key.clone();
                        debug!("Patching Data: {} on {}", json.to_string(), k);
                        v1_map.insert(key, data);
                        let json: serde_json::Value = v1_map.get(&k).unwrap().clone().into();
                        debug!("Patched Data: {} on {}", json.to_string(), k);
                    } else if object.contains_key(&key) {
                        debug!("SchemaValue does not have V1");
                        let json: serde_json::Value = data.clone().into();
                        let k = key.clone();
                        debug!("Patching Data: {} on {}", json.to_string(), k);
                        object.insert(key, data);
                        let json: serde_json::Value = object.get(&k).unwrap().clone().into();
                        debug!("Patched Data: {} on {}", json.to_string(), k);
                    }
                }

                db_data.raw_data = Set(schema_value.try_to_vec()?);
                debug!("SchemaValue serialized");
                let parsed_data: serde_json::Value = schema_value.into();
                debug!("Complete Data After Patch: {}", parsed_data.to_string());
                db_data.parsed_data = Set(parsed_data);
                debug!("Data updated in object");

                let query: Statement = compressed_data::Entity::update(db_data)
                    .filter(compressed_data::Column::TreeId.eq(tree_id.to_vec()))
                    .filter(compressed_data::Column::LeafIdx.eq(leaf_idx as i64))
                    .build(DbBackend::Postgres);

                debug!(
                    "Query builed successfully, {}, values {:#?}",
                    query.sql, query.values
                );
                txn.execute(query)
                    .await
                    .map_err(|db_err| IngesterError::StorageWriteError(db_err.to_string()))?;
                debug!("Query executed successfully");
            }
            _ => {}
        },
    }
    Ok(())
}
