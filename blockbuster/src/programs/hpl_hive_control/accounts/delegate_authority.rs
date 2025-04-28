use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct DelegateAuthority {
    /// Bump value used for PDA.
    pub bump: u8,

    /// Public key of the project associated with this delegated authority.
    pub project: Pubkey,

    /// Public key of the authority getting these permissions.
    pub authority: Pubkey,

    /// List of service delegations, each specifying the program and its permissions.
    pub delegations: Vec<ServiceDelegation>,
}
impl DelegateAuthority {
    pub const DISCRIMINATOR: [u8; 8] = [121, 110, 250, 77, 147, 244, 126, 81];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum ServiceDelegation {
    /// Hive Control delegation with a specific set of permissions.
    HiveControl {
        /// The permissions granted to the Hive Control.
        permission: HiveControlPermission,
    },

    /// Asset Assembler program delegation with a specific set of permissions and an associated index.
    CharacterManager {
        /// Index of the service in the service vector in the project struct.
        index: u8,

        /// The permissions granted to the asset Assembler programs.
        permission: CharacterManagerPermissions,
    },

    /// Resource Manager program delegation with a specific set of permissions and an associated index.
    ResourceManager {
        /// The permissions granted to the Resource Manager programs.
        permission: ResourceManagerPermission,
    },

    /// Nectar Staking program delegation with a specific set of permissions and an associated index.
    NectarStaking {
        /// Index of the service in the service vector in the project struct.
        index: u8,

        /// The permissions granted to the Nectar staking programs.
        permission: NectarStakingPermission,
    },

    /// Nectar Missions program delegation with a specific set of permissions and an associated index.
    NectarMissions {
        /// Index of the service in the service vector in the project struct.
        index: u8,

        /// The permissions granted to the Nectar Missions programs.
        permission: NectarMissionsPermission,
    },

    /// Buzz Guild program delegation with a specific set of permissions and an associated index.
    BuzzGuild {
        /// Index of the service in the service vector in the project struct.
        index: u8,

        /// The permissions granted to the Buzz Guild programs.
        permission: BuzzGuildPermission,
    },
}

/// Enum representing different types of permissions for the master program delegation.
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq, Eq)]
pub enum HiveControlPermission {
    /// Permission to manage project driver.
    ManageProjectDriver,

    /// Permission to manage criterias i.e, collections and creators.
    ManageCriterias,

    /// Permission to manage services.
    ManageServices,

    /// Permission to update platform data.
    UpdatePlatformData,
}

/// Enum representing different types of permissions for the asset assembler program delegation.
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq, Eq)]
pub enum CharacterManagerPermissions {
    /// Permission to manage assembler config.
    ManageAssemblerConfig,

    /// Permission to manage character models.
    ManageCharacterModels,

    /// Permissions to assign traits to a character
    AssignCharacterTraits,
}

/// Enum representing different types of permissions for the resource manager delegation.
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq, Eq)]
pub enum ResourceManagerPermission {
    /// Permission to manage resources.
    CreateResources,

    /// Permission to mint resources.
    MintResources,

    /// Permission to manage currency status.
    BurnResources,

    /// Permission to manager faucet.
    CreateFaucet,

    /// Permission to manage recipes.
    CreateRecipe,
}

/// Enum representing different types of permissions for the Nectar staking delegation.
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq, Eq)]
pub enum NectarStakingPermission {
    /// Permission to manage the staking pool.
    ManageStakingPool,

    /// Permission to withdraw staking pool rewards.
    WithdrawStakingPoolRewards,

    /// Permission to manage the SPL staking pool.
    ManageSplStakingPool,

    /// Permission to withdraw SPL staking pool rewards.
    WithdrawSplStakingPoolRewards,
}

/// Enum representing different types of permissions for the Nectar missions delegation.
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq, Eq)]
pub enum NectarMissionsPermission {
    /// Permission to manage the mission pool.
    ManageMissionPool,

    /// Permission to withdraw mission pool rewards.
    WithdrawMissionPoolRewards,
}

/// Enum representing different types of permissions for the Buzz guild delegation.
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq, Eq)]
pub enum BuzzGuildPermission {
    /// Permission to manage the guild kits.
    ManageGuildKit,
}
