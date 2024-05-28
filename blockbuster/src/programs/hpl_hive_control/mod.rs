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
    hpl_hive_control,
    "HivezrprVqHR6APKKQkkLHmUG8waZorXexEBRZWh5LRm"
);

#[derive(Clone, PartialEq)]
pub enum HplHiveControlAccount {
    Uninitialized,
    Unknown,
    Global(accounts::Global),
    Project(accounts::Project),
    DelegateAuthority(accounts::DelegateAuthority),
}

impl ParseResult for HplHiveControlAccount {
    fn result(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
    fn result_type(&self) -> ProgramParseResult {
        ProgramParseResult::HplHiveControl(self)
    }
}

pub struct HplHiveControlParser;

impl ProgramParser for HplHiveControlParser {
    fn key(&self) -> Pubkey {
        hpl_hive_control()
    }
    fn key_match(&self, key: &Pubkey) -> bool {
        key == &hpl_hive_control()
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
            return Ok(Box::new(HplHiveControlAccount::Uninitialized));
        }
        let mut discriminator = [0; 8];
        discriminator.copy_from_slice(&account_data[..8]);
        Ok(Box::new(match discriminator {
            accounts::Global::DISCRIMINATOR => HplHiveControlAccount::Global(
                accounts::Global::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            accounts::Project::DISCRIMINATOR => HplHiveControlAccount::Project(
                accounts::Project::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            accounts::DelegateAuthority::DISCRIMINATOR => HplHiveControlAccount::DelegateAuthority(
                accounts::DelegateAuthority::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            _ => HplHiveControlAccount::Unknown,
        }))
    }
}
