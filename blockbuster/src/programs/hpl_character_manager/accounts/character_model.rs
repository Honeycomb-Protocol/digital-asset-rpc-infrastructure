use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct CharacterModel {
    pub bump: u8,
    pub key: Pubkey,
    pub project: Pubkey,
    pub config: CharacterConfig,
    pub attributes: Schema,
    pub merkle_trees: ControlledMerkleTrees,
}
impl CharacterModel {
    pub const DISCRIMINATOR: [u8; 8] = [48, 232, 95, 182, 18, 16, 71, 113];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum CharacterConfig {
    Wrapped(Vec<AssetCriteria>),
    Assembled {
        assembler_config: Pubkey,
        name: String,
        symbol: String,
        description: String,
        creators: Vec<NftCreator>,
        seller_fee_basis_points: u16,
        collection_name: String,
        mint_as: MintAs,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, ToSchema, PartialEq)]
pub enum AssetCriteria {
    Prepopulated,
    Collection(Pubkey),
    Creator(Pubkey),
    MerkleTree(Pubkey),
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct NftCreator {
    pub address: Pubkey,
    pub share: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum MintAs {
    MplCore,
    MplMetadata,
    MplBubblegum { merkle_tree: Pubkey },
    TokenExtensions,
}
