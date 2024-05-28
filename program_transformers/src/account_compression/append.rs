use crate::bubblegum::insert_change_log;
use crate::error::{ProgramTransformerError, ProgramTransformerResult};
use blockbuster::{
    instruction::InstructionBundle, programs::account_compression::AccountCompressionInstruction,
};
use sea_orm::{ConnectionTrait, TransactionTrait};
// TODO -> consider moving structs into these functions to avoid clone

pub async fn append<'c, T>(
    parsing_result: &AccountCompressionInstruction,
    bundle: &InstructionBundle<'c>,
    txn: &'c T,
    cl_audits: bool,
) -> ProgramTransformerResult<()>
where
    T: ConnectionTrait + TransactionTrait,
{
    if let Some(cl) = &parsing_result.tree_update {
        return insert_change_log(
            cl,
            bundle.slot,
            bundle.txn_id,
            txn,
            "LowLevelAppend",
            cl_audits,
        )
        .await;
    }
    Err(ProgramTransformerError::ParsingError(
        "Ix not parsed correctly".to_string(),
    ))
}
