use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Faucet {
    // the project that owns this faucet
    pub project: Pubkey,

    // the resource that this faucet is going to dispense
    pub resource: Pubkey,

    // the amount of resource this faucet dispenses
    pub amount: u64,

    // the interval at which this faucet dispenses resources
    pub repeat_interval: i64,

    // last time this faucet dispensed resources
    pub last_claimed: i64,
}
impl Faucet {
    pub const DISCRIMINATOR: [u8; 8] = [146, 11, 249, 142, 199, 197, 61, 0];
}
