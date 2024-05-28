use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct AssemblerConfig {
    pub bump: u8,
    pub ticker: String,
    pub project: Pubkey,
    pub order: Vec<String>,
    pub merkle_trees: ControlledMerkleTrees,
}

impl AssemblerConfig {
    pub const DISCRIMINATOR: [u8; 8] = [5, 4, 69, 145, 53, 127, 224, 177];
}
