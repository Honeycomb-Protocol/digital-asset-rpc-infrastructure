use anchor_lang::{prelude::*, solana_program::keccak};
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct StakingPool {
    pub bump: u8,
    pub project: Pubkey,
    pub key: Pubkey,
    pub resource: Pubkey,
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

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]

pub struct SplStakingPool {
    /// The PDA bump to derive the staking pool address
    pub bump: u8,

    pub nonce: u16,

    /// The public key representing the project associated with this staking pool
    pub project: Pubkey,

    /// name of the staking_pool
    pub name: ShortString,

    /// The mint address of the token to be staked by users
    pub stake_token_mint: Pubkey,

    /// The minimum duration (in seconds) that tokens must be staked to qualify for rewards (e.g., 86400 for one day)
    pub min_stake_duration_secs: Option<u64>,

    /// The maximum duration (in seconds) for which rewards can be distributed (e.g., 31536000 for one year)
    pub max_stake_duration_secs: Option<u64>,

    /// The unix_timestamp when the staking starts
    pub start_time: Option<i64>,

    /// The unix_timestamp when the staking ends
    pub end_time: Option<i64>,

    /// The total amount of tokens currently staked in the staking pool
    pub total_staked_amount: u64,

    /// Pool Multipliers
    pub multipliers: ShortVec<SplMultiplier>,

    /// Vector of reward pool configurations
    pub reward_config: RewardPoolConfig,

    /// Controlled Merkle Tree for the Reciept data structure
    pub merkle_trees: ControlledMerkleTrees,

    // padding
    _padding: [u8; 4],
}

impl SplStakingPool {
    pub const DISCRIMINATOR: [u8; 8] = [102, 74, 25, 250, 108, 54, 144, 39];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum RewardPoolConfig {
    NotSet,
    ApyConfig {
        /// The mint address of the reward token
        reward_token_mint: Pubkey,

        /// The token account to store the reward SPL tokens for distribution
        reward_vault: Option<Pubkey>,

        /// The rewards per selected duration
        rewards_per_duration: u64,

        /// The duration of the rewards in seconds
        rewards_duration: u64,

        /// The total number of reward tokens allocated for this reward pool
        total_reward_amount: Option<u64>,
    },
    StakeWeightConfig {
        pools: ShortVec<StakeWeightConfig>,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq, ToNode)]
pub struct StakeWeightConfig {
    weight: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq, ToNode)]
pub struct SplMultiplier {
    pub value: u16,
    pub multiplier_type: SplMultiplierType,
}
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq, ToNode)]
pub enum SplMultiplierType {
    /// The multiplier is applied to the stake duration
    StakeDuration { min_duration: u64 },

    /// The multiplier is applied based on the stake amount
    StakeAmount { min_amount: u64 },
}

