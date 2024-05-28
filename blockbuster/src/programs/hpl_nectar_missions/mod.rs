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
    hpl_nectar_missions_id,
    "HuntaX1CmUt5EByyFPE8pMf13SpvezybmMTtjmpmGmfj"
);

#[derive(Clone, PartialEq)]
pub enum HplNectarMissionsAccount {
    Uninitialized,
    Unknown,
    MissionPool(accounts::MissionPool),
    Mission(accounts::Mission),
}

impl ParseResult for HplNectarMissionsAccount {
    fn result(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
    fn result_type(&self) -> ProgramParseResult {
        ProgramParseResult::HplNectarMissions(self)
    }
}

pub struct HplNectarMissionsParser;

impl ProgramParser for HplNectarMissionsParser {
    fn key(&self) -> Pubkey {
        hpl_nectar_missions_id()
    }
    fn key_match(&self, key: &Pubkey) -> bool {
        key == &hpl_nectar_missions_id()
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
            return Ok(Box::new(HplNectarMissionsAccount::Uninitialized));
        }
        let mut discriminator = [0; 8];
        discriminator.copy_from_slice(&account_data[..8]);
        Ok(Box::new(match discriminator {
            accounts::MissionPool::DISCRIMINATOR => HplNectarMissionsAccount::MissionPool(
                accounts::MissionPool::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            accounts::Mission::DISCRIMINATOR => HplNectarMissionsAccount::Mission(
                accounts::Mission::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            _ => HplNectarMissionsAccount::Unknown,
        }))
    }
}
