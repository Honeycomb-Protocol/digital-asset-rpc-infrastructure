use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Staker {
    pub bump: u8,
    pub staking_pool: Pubkey,
    pub wallet: Pubkey,
    pub total_staked: u64,
}
impl Staker {
    pub const DISCRIMINATOR: [u8; 8] = [171, 229, 193, 85, 67, 177, 151, 4];
}
