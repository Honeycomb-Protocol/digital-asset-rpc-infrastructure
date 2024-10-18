use super::AssetCriteria;
use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct AssetCustody {
    pub bump: u8,

    /// Where this character came from
    pub wallet: Pubkey,

    pub character_model: Option<Pubkey>,
    pub source: Option<CharacterSource>,
    pub character: Option<CharacterAssetConfig>,
}

impl AssetCustody {
    pub const DISCRIMINATOR: [u8; 8] = [214, 130, 16, 11, 2, 108, 220, 26];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]

pub enum CharacterSource {
    Wrapped {
        mint: Pubkey,
        criteria: AssetCriteria,
        is_compressed: bool,
    },
    Assembled {
        hash: Pubkey,
        mint: Pubkey,
        uri: String,
        attributes: VecMap<String, String>, // label, name
        update_authority: Pubkey,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct CharacterAssetConfig {
    pub tree: Pubkey,
    pub leaf: u32,
}
