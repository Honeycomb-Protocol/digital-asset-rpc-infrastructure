use {
    crate::{
        error::{ProgramTransformerError, ProgramTransformerResult},
        AccountInfo,
    },
    blockbuster::programs::hpl_resource_manager::{
        Faucet, HplResourceManagerAccount, Recipe, Resource,
    },
    sea_orm::DatabaseConnection,
};

pub async fn handle_hpl_resource_manager_account<'a, 'b, 'c>(
    account_info: &AccountInfo,
    parsing_result: &'a HplResourceManagerAccount,
    db: &'b DatabaseConnection,
) -> ProgramTransformerResult<()> {
    match &parsing_result {
        HplResourceManagerAccount::Resource(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                Resource::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        HplResourceManagerAccount::Recipe(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                Recipe::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        HplResourceManagerAccount::Faucet(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                Faucet::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        _ => Err(ProgramTransformerError::NotImplemented),
    }?;
    Ok(())
}
