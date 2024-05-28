use crate::error::ProgramTransformerResult;
use blockbuster::{
    instruction::InstructionBundle, programs::account_compression::AccountCompressionInstruction,
};
use sea_orm::{ConnectionTrait, TransactionTrait};
// TODO -> consider moving structs into these functions to avoid clone

pub async fn verify_leaf<'c, T>(
    _parsing_result: &AccountCompressionInstruction,
    _bundle: &InstructionBundle<'c>,
    _txn: &'c T,
    _cl_audits: bool,
) -> ProgramTransformerResult<()>
where
    T: ConnectionTrait + TransactionTrait,
{
    Ok(())
}
