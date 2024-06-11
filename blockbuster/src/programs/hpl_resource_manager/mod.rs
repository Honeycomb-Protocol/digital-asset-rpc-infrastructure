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
    hpl_resource_manager_id,
    "RSCR7UoY65mDMK8z2eCBvFmj4HSepGEY9ZjdCTiUDUA"
);

#[derive(Clone, PartialEq)]
pub enum HplResourceManagerAccount {
    Uninitialized,
    Unknown,
    Resource(accounts::Resource),
    Recipe(accounts::Recipe),
    Faucet(accounts::Faucet),
}

impl ParseResult for HplResourceManagerAccount {
    fn result(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
    fn result_type(&self) -> ProgramParseResult {
        ProgramParseResult::HplResourceManager(self)
    }
}

pub struct HplResourceManagerParser;

impl ProgramParser for HplResourceManagerParser {
    fn key(&self) -> Pubkey {
        hpl_resource_manager_id()
    }
    fn key_match(&self, key: &Pubkey) -> bool {
        key == &hpl_resource_manager_id()
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
            return Ok(Box::new(HplResourceManagerAccount::Uninitialized));
        }
        let mut discriminator = [0; 8];
        discriminator.copy_from_slice(&account_data[..8]);
        Ok(Box::new(match discriminator {
            accounts::Resource::DISCRIMINATOR => HplResourceManagerAccount::Resource(
                accounts::Resource::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            accounts::Faucet::DISCRIMINATOR => HplResourceManagerAccount::Faucet(
                accounts::Faucet::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            accounts::Recipe::DISCRIMINATOR => HplResourceManagerAccount::Recipe(
                accounts::Recipe::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            _ => HplResourceManagerAccount::Unknown,
        }))
    }
}
