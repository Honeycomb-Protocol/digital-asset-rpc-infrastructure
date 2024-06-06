use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Recipe {
    pub bump: u8,

    pub project: Pubkey,

    pub key: Pubkey,

    pub xp: XpPair,

    pub output: ResourceAmountPair,

    pub inputs: Vec<ResourceAmountPair>,

    pub output_characteristics: VecMap<String, String>,
}
impl Recipe {
    pub const DISCRIMINATOR: [u8; 8] = [10, 162, 156, 100, 56, 193, 205, 77];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct ResourceAmountPair {
    pub resource: Pubkey,

    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct XpPair {
    pub label: String,

    pub increment: u64,
}
