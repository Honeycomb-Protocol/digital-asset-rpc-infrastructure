use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Mission {
    pub bump: u8,
    pub project: Pubkey,
    pub mission_pool: Pubkey,
    pub name: String,
    pub min_xp: u64,
    pub cost: MissionCost,
    pub requirement: MissionRequirement,
    pub rewards: Vec<Reward>,
}

impl Mission {
    pub const DISCRIMINATOR: [u8; 8] = [170, 56, 116, 75, 24, 11, 109, 12];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct MissionCost {
    pub amount: u64,
    pub resource_address: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum MissionRequirement {
    Time { duration: u64 },
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Reward {
    pub min: u64,
    pub max: u64,
    pub reward_type: RewardType,
}
impl Reward {
    pub const LEN: usize = 8 + 56;
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum RewardType {
    Xp,
    Resource { address: Pubkey },
}
