use {
    crate::{
        error::{ProgramTransformerError, ProgramTransformerResult},
        AccountInfo,
    },
    blockbuster::programs::hpl_character_manager::{
        AssemblerConfig, AssetCustody, CharacterModel, HplCharacterManagerAccount,
    },
    sea_orm::DatabaseConnection,
};

pub async fn handle_hpl_character_manager_account<'a, 'b, 'c>(
    account_info: &AccountInfo,
    parsing_result: &'a HplCharacterManagerAccount,
    db: &'b DatabaseConnection,
) -> ProgramTransformerResult<()> {
    match &parsing_result {
        HplCharacterManagerAccount::AssemblerConfig(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                AssemblerConfig::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        HplCharacterManagerAccount::CharacterModel(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                CharacterModel::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        HplCharacterManagerAccount::AssetCustody(account) => {
            super::save_account(
                db,
                account_info.pubkey.to_bytes().to_vec(),
                account_info.owner.to_bytes().to_vec(),
                AssetCustody::DISCRIMINATOR.to_vec(),
                account,
                account_info.slot as i64,
            )
            .await
        }
        _ => Err(ProgramTransformerError::NotImplemented),
    }?;
    Ok(())
}
