use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct AssetCustody {
    pub bump: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub kind: SourceKind,
    pub collection: Option<Pubkey>,
    pub merkle_tree: Option<Pubkey>,
    pub creators: Option<Vec<Pubkey>>,
    pub uri: Option<String>,
    pub character_model: Pubkey,
}

impl AssetCustody {
    pub const DISCRIMINATOR: [u8; 8] = [214, 130, 16, 11, 2, 108, 220, 26];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum SourceKind {
    MplMetadata,
    MplBubblegum,
    TokenExtensions,
    MplCoreAsset,
}
