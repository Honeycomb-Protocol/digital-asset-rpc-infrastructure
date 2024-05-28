use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct DelegateAuthority {
    pub bump: u8,
    pub project: Pubkey,
    pub authority: Pubkey,
    pub delegations: Vec<ServiceDelegation>,
}
impl DelegateAuthority {
    pub const DISCRIMINATOR: [u8; 8] = [121, 110, 250, 77, 147, 244, 126, 81];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum ServiceDelegation {
    HiveControl {
        permission: HiveControlPermission,
    },
    AssetAssembler {
        index: u8,
        permission: AssetAssemblerPermission,
    },
    AssetManager {
        index: u8,
        permission: AssetManagerPermission,
    },
    CurrencyManager {
        permission: CurrencyManagerPermission,
    },
    NectarStaking {
        index: u8,
        permission: NectarStakingPermission,
    },
    NectarMissions {
        index: u8,
        permission: NectarMissionsPermission,
    },
    BuzzGuild {
        index: u8,
        permission: BuzzGuildPermission,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum HiveControlPermission {
    ManageCriterias,
    ManageServices,
    ManageIndexing,
    ManageProfiles,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum AssetAssemblerPermission {
    ManageAssembler,
    UpdateBlock,
    UpdateBlockDefinition,
    UpdateNFT,
    InitialArtGeneration,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum AssetManagerPermission {
    ManageAssets,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum CurrencyManagerPermission {
    ManageCurrencies,
    MintCurrencies,
    ManageCurrencyStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum NectarStakingPermission {
    ManageStakingPool,
    WithdrawStakingPoolRewards,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum NectarMissionsPermission {
    ManageMissionPool,
    WithdrawMissionPoolRewards,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum BuzzGuildPermission {
    ManageGuildKit,
}
