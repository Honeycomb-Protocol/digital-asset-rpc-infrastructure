use {
    crate::{
        error::{ProgramTransformerError, ProgramTransformerResult},
        AccountInfo,
    },
    blockbuster::programs::hpl_currency_manager::{
        Currency, HolderAccount, HplCurrencyManagerAccount,
    },
    sea_orm::DatabaseConnection,
};

pub async fn handle_hpl_currency_manager_account<'a, 'b, 'c>(
    account_info: &AccountInfo,
    parsing_result: &'a HplCurrencyManagerAccount,
    db: &'b DatabaseConnection,
) -> ProgramTransformerResult<()> {
    match &parsing_result {
        HplCurrencyManagerAccount::Currency(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                Currency::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        HplCurrencyManagerAccount::HolderAccount(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                HolderAccount::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }

        _ => Err(ProgramTransformerError::NotImplemented),
    }?;
    Ok(())
}
