use crate::{
    error::BlockbusterError,
    program_handler::{ParseResult, ProgramParser},
    programs::ProgramParseResult,
};
use borsh::BorshDeserialize;
use solana_sdk::{pubkey::Pubkey, pubkeys};

mod accounts;

pub use accounts::*;

pubkeys!(
    hpl_character_manager_id,
    "ChRCtrG7X5kb9YncA4wuyD68DXXL8Szt3zBCCGiioBTg"
);

#[derive(Clone, PartialEq)]
pub enum HplCharacterManagerAccount {
    Uninitialized,
    Unknown,
    AssemblerConfig(accounts::AssemblerConfig),
    CharacterModel(accounts::CharacterModel),
    AssetCustody(accounts::AssetCustody),
}

impl ParseResult for HplCharacterManagerAccount {
    fn result(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
    fn result_type(&self) -> ProgramParseResult {
        ProgramParseResult::HplCharacterManager(self)
    }
}

pub struct HplCharacterManagerParser;

impl ProgramParser for HplCharacterManagerParser {
    fn key(&self) -> Pubkey {
        hpl_character_manager_id()
    }
    fn key_match(&self, key: &Pubkey) -> bool {
        key == &hpl_character_manager_id()
    }

    fn handles_account_updates(&self) -> bool {
        true
    }

    fn handles_instructions(&self) -> bool {
        false
    }

    fn handle_account(
        &self,
        account_data: &[u8],
    ) -> Result<Box<(dyn ParseResult + 'static)>, BlockbusterError> {
        if account_data.is_empty() {
            return Ok(Box::new(HplCharacterManagerAccount::Uninitialized));
        }
        let mut discriminator = [0; 8];
        discriminator.copy_from_slice(&account_data[..8]);
        Ok(Box::new(match discriminator {
            accounts::AssemblerConfig::DISCRIMINATOR => {
                HplCharacterManagerAccount::AssemblerConfig(
                    accounts::AssemblerConfig::deserialize(&mut &account_data[8..])
                        .map_err(|_| BlockbusterError::DeserializationError)?,
                )
            }
            accounts::CharacterModel::DISCRIMINATOR => HplCharacterManagerAccount::CharacterModel(
                accounts::CharacterModel::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            accounts::AssetCustody::DISCRIMINATOR => HplCharacterManagerAccount::AssetCustody(
                accounts::AssetCustody::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            _ => HplCharacterManagerAccount::Unknown,
        }))
    }
}
