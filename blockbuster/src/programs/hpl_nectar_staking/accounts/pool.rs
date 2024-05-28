use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct StakingPool {
    pub bump: u8,
    pub project: Pubkey,
    pub key: Pubkey,
    pub currency: Pubkey,
    pub lock_type: LockType,
    pub name: String,
    pub rewards_per_duration: u64,
    pub rewards_duration: u64,
    pub max_rewards_duration: Option<u64>,
    pub min_stake_duration: Option<u64>,
    pub cooldown_duration: Option<u64>,
    pub reset_stake_duration: bool,
    pub allowed_mints: bool,
    pub total_staked: u64,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub character_models: Vec<Pubkey>,
}
impl StakingPool {
    pub const DISCRIMINATOR: [u8; 8] = [203, 19, 214, 220, 220, 154, 24, 102];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum LockType {
    Freeze,
    Custoday,
}
