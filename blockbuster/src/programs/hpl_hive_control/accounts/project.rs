use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;
use spl_account_compression::Node;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Project {
    pub bump: u8,
    pub authority: Pubkey,
    pub key: Pubkey,
    pub driver: Pubkey,
    pub name: String,
    pub services: Vec<Service>,
    pub associated_programs: Vec<AssociatedProgram>,
    pub profile_data_config: ProfileDataConfig,
    pub profile_trees: ControlledMerkleTrees,
    pub badge_criteria: Option<Vec<BadgeCriteria>>,
    pub subsidy_fees: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct BadgeCriteria {
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub index: u16,
    pub condition: BadgeCriteriaCondition,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum BadgeCriteriaCondition {
    Public,
    Whitelisted { root: Node },
}
impl Project {
    pub const DISCRIMINATOR: [u8; 8] = [205, 168, 189, 202, 181, 247, 142, 19];
}
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct ProfileDataConfig {
    pub achievements: Vec<String>,
    pub custom_data_fields: Vec<String>,
}
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum Service {
    Assembler { assembler_id: Pubkey },
    AssetManager { asset_manager_id: Pubkey },
    Paywall,
    Staking { pool_id: Pubkey },
    Missions { pool_id: Pubkey },
    Raffles { pool_id: Pubkey },
    GuildKit { kit_id: Pubkey },
    GameState,
    MatchMaking,
}
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct AssociatedProgram {
    pub address: Pubkey,
    pub trusted_actions: Vec<SerializableActions>,
}
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum SerializableActions {
    // General Actions
    FeeExempt,
    PublicLow,
    PublicHigh,
    DriverAction,

    // Hive Control Actions
    CreateProject,
    ManageCriterias,
    AddService,
    RemoveService,
    ManageIndexing,
    ManageDelegateAuthority,
    ManageProfiles,

    // Asset Assembler Actions
    ManageAssembler,

    // Asset Manager Actions
    ManageAssets,

    // Currency Manager Actions
    ManageCurrencies,
    MintCurrencies,
    ManageCurrencyStatus,

    // Nectar Staking Actions
    ManageStakingPool,
    WithdrawStakingPoolRewards,

    // Nectar Missions Actions
    ManageMissionPool,
    WithdrawMissionPoolRewards,

    // Buzz Guild Kit Actions
    ManageGuildKit,
}
