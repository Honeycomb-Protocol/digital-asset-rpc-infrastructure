use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Recipe {
    pub bump: u8,

    pub project: Pubkey,

    pub key: Pubkey,

    pub xp: u64,

    pub ingredients: Vec<Ingredient>,

    pub meal: Meal,
}
impl Recipe {
    pub const DISCRIMINATOR: [u8; 8] = [10, 162, 156, 100, 56, 193, 205, 77];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Ingredient {
    pub resource_address: Pubkey,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Meal {
    pub resource_address: Pubkey,
    pub amount: u64,
}
