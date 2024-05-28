use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Multipliers {
    pub bump: u8,
    pub staking_pool: Pubkey,
    pub decimals: u8,
    pub duration_multipliers: Vec<Multiplier>,
    pub count_multipliers: Vec<Multiplier>,
    pub creator_multipliers: Vec<Multiplier>,
    pub collection_multipliers: Vec<Multiplier>,
}
impl Multipliers {
    pub const DISCRIMINATOR: [u8; 8] = [129, 233, 66, 228, 168, 129, 38, 204];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Multiplier {
    pub value: u64,
    pub multiplier_type: MultiplierType,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum MultiplierType {
    StakeDuration { min_duration: u64 },
    NFTCount { min_count: u64 },
    Creator { creator: Pubkey },
    Collection { collection: Pubkey },
}
