use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Resource {
    /// Bump seed for the PDA
    pub bump: u8,

    /// The project this resource is associated with
    pub project: Pubkey,

    /// The mint of this resource
    pub mint: Pubkey,

    /// storage of the resource
    pub storage: ResourceStorage,

    // the characteristics of this resource
    pub kind: ResourceKind,
}
impl Resource {
    pub const DISCRIMINATOR: [u8; 8] = [10, 160, 2, 1, 42, 207, 51, 212];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum ResourceStorage {
    AccountState,
    LedgerState {
        merkle_trees: ControlledMerkleTrees,
        promise_supply: u64,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum ResourceKind {
    Exported,

    HplFungible {
        decimals: u8,
    },

    HplNonFungible {
        characteristics: Vec<String>,
    },

    WrappedFungible {
        decimals: u8,
        custody: ResourceCustody,
    },

    WrappedMplCore {
        characteristics: Vec<String>,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum ResourceCustody {
    Authority,
    Supply { burner_destination: Option<Pubkey> },
}
