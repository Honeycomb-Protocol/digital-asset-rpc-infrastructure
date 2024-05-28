use {
    crate::{
        error::{ProgramTransformerError, ProgramTransformerResult},
        AccountInfo,
    },
    blockbuster::programs::hpl_hive_control::{
        DelegateAuthority, Global, HplHiveControlAccount, Project,
    },
    sea_orm::DatabaseConnection,
};

pub async fn handle_hpl_hive_control_account<'a, 'b, 'c>(
    account_info: &AccountInfo,
    parsing_result: &'a HplHiveControlAccount,
    db: &'b DatabaseConnection,
) -> ProgramTransformerResult<()> {
    match &parsing_result {
        HplHiveControlAccount::Global(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                Global::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        HplHiveControlAccount::Project(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                Project::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        HplHiveControlAccount::DelegateAuthority(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                DelegateAuthority::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        _ => Err(ProgramTransformerError::NotImplemented),
    }?;
    Ok(())
}
