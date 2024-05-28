use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Global {
    pub bump: u8,
    pub config: VecMap<String, Config>,
    pub user_trees: ControlledMerkleTrees,
    pub total_users: u64,
}
impl Global {
    pub const DISCRIMINATOR: [u8; 8] = [167, 232, 232, 177, 200, 108, 114, 127];
}
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum Config {
    SingleValue(ConfigValue),
    MultiValue(Vec<ConfigValue>),
}
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum ConfigValue {
    String(String),
    Pubkey(Pubkey),
}
