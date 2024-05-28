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
    hpl_nectar_staking_id,
    "MiNESdRXUSmWY7NkAKdW9nMkjJZCaucguY3MDvkSmr6"
);

#[derive(Clone, PartialEq)]
pub enum HplNectarStakingAccount {
    Uninitialized,
    Unknown,
    StakingPool(accounts::StakingPool),
    Multipliers(accounts::Multipliers),
    Staker(accounts::Staker),
}

impl ParseResult for HplNectarStakingAccount {
    fn result(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
    fn result_type(&self) -> ProgramParseResult {
        ProgramParseResult::HplNectarStaking(self)
    }
}

pub struct HplNectarStakingParser;

impl ProgramParser for HplNectarStakingParser {
    fn key(&self) -> Pubkey {
        hpl_nectar_staking_id()
    }
    fn key_match(&self, key: &Pubkey) -> bool {
        key == &hpl_nectar_staking_id()
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
            return Ok(Box::new(HplNectarStakingAccount::Uninitialized));
        }
        let mut discriminator = [0; 8];
        discriminator.copy_from_slice(&account_data[..8]);
        Ok(Box::new(match discriminator {
            accounts::StakingPool::DISCRIMINATOR => HplNectarStakingAccount::StakingPool(
                accounts::StakingPool::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            accounts::Multipliers::DISCRIMINATOR => HplNectarStakingAccount::Multipliers(
                accounts::Multipliers::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            accounts::Staker::DISCRIMINATOR => HplNectarStakingAccount::Staker(
                accounts::Staker::deserialize(&mut &account_data[8..])
                    .map_err(|_| BlockbusterError::DeserializationError)?,
            ),
            _ => HplNectarStakingAccount::Unknown,
        }))
    }
}
