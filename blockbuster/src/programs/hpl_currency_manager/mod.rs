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
    hpl_currency_manager_id,
    "CrncyaGmZfWvpxRcpHEkSrqeeyQsdn4MAedo9KuARAc4"
);

#[derive(Clone, PartialEq)]
pub enum HplCurrencyManagerAccount {
    Uninitialized,
    Unknown,
    Currency(accounts::Currency),
    HolderAccount(accounts::HolderAccount),
}

impl ParseResult for HplCurrencyManagerAccount {
    fn result(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
    fn result_type(&self) -> ProgramParseResult {
        ProgramParseResult::HplCurrencyManager(self)
    }
}

pub struct HplCurrencyManagerParser;

impl ProgramParser for HplCurrencyManagerParser {
    fn key(&self) -> Pubkey {
        hpl_currency_manager_id()
    }
    fn key_match(&self, key: &Pubkey) -> bool {
        key == &hpl_currency_manager_id()
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
            return Ok(Box::new(HplCurrencyManagerAccount::Uninitialized));
        }
        let mut discriminator = [0; 8];
        discriminator.copy_from_slice(&account_data[..8]);
        Ok(Box::new(match discriminator {
            accounts::Currency::DISCRIMINATOR => HplCurrencyManagerAccount::Currency(
                accounts::Currency::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            accounts::HolderAccount::DISCRIMINATOR => HplCurrencyManagerAccount::HolderAccount(
                accounts::HolderAccount::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),

            _ => HplCurrencyManagerAccount::Unknown,
        }))
    }
}
