use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct MissionPool {
    pub bump: u8,
    pub project: Pubkey,
    pub name: String,
    pub factions_merkle_root: [u8; 32],
    pub randomizer_round: u8,
    pub character_models: Vec<Pubkey>,
    pub guild_kits: Vec<u8>,
}
impl MissionPool {
    pub const DISCRIMINATOR: [u8; 8] = [106, 55, 99, 194, 178, 110, 104, 188];
}
