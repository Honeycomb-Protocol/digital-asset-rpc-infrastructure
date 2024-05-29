use anchor_lang::prelude::*;
use hpl_toolkit::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct HolderAccount {
    pub bump: u8,
    /// The currency associated with this account
    pub currency: Pubkey,
    /// The owner of this account.
    pub owner: Pubkey,
    /// The token account that holds the currency.
    pub token_account: Pubkey,
    /// Holder status
    pub status: HolderStatus,
    /// When this holder account was created
    pub created_at: i64,
}
impl HolderAccount {
    pub const DISCRIMINATOR: [u8; 8] = [164, 95, 70, 248, 145, 238, 169, 176];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum HolderStatus {
    /// The holder is active and can be used to send and receive currency.
    Active,
    /// The holder is inactive and cannot be used to send or receive currency.
    Inactive,
}
