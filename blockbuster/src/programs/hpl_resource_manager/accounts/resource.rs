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

    pub metadata: ResourceMetadataArgs,

    /// token account trees
    pub merkle_trees: ControlledMerkleTrees,

    // the characteristics of this resource
    pub kind: ResourceKind,
}
impl Resource {
    pub const DISCRIMINATOR: [u8; 8] = [10, 160, 2, 1, 42, 207, 51, 212];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum ResourceKind {
    Fungible {
        decimals: u8,
    },

    INF {
        characteristics: Vec<String>,
        supply: u32,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct ResourceMetadataArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}
