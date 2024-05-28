use {
    crate::{
        error::{ProgramTransformerError, ProgramTransformerResult},
        AccountInfo,
    },
    blockbuster::programs::hpl_nectar_staking::{
        HplNectarStakingAccount, Multipliers, Staker, StakingPool,
    },
    sea_orm::DatabaseConnection,
};

pub async fn handle_hpl_nectar_staking_account<'a, 'b, 'c>(
    account_info: &AccountInfo,
    parsing_result: &'a HplNectarStakingAccount,
    db: &'b DatabaseConnection,
) -> ProgramTransformerResult<()> {
    match &parsing_result {
        HplNectarStakingAccount::StakingPool(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                StakingPool::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        HplNectarStakingAccount::Multipliers(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                Multipliers::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        HplNectarStakingAccount::Staker(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                Staker::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        _ => Err(ProgramTransformerError::NotImplemented),
    }?;
    Ok(())
}
