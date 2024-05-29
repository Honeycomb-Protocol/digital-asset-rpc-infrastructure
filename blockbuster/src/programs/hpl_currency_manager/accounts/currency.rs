use {anchor_lang::prelude::*, hpl_toolkit::prelude::*};

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub struct Currency {
    pub bump: u8,

    /// The project this currency is associated with.
    pub project: Pubkey,

    /// The spl mint of the currency.
    pub mint: Pubkey,

    /// The type of currency.
    pub kind: CurrencyKind,

    /// Transaction Hook
    pub tx_hook: TxHook,
}
impl Currency {
    pub const DISCRIMINATOR: [u8; 8] = [191, 62, 116, 219, 163, 67, 229, 200];
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum TxHook {
    /// No hook
    User,
    /// Hook to a program
    Authority,
    /// Hook to a program
    CPIProgram {
        /// The program id
        program_id: Pubkey,
        /// The data to be passed to the program
        data: Vec<u8>,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum CurrencyKind {
    /// Represents a permissioned currency, further specified by `PermissionedCurrencyKind`.
    Permissioned { kind: PermissionedCurrencyKind },
    /// Represents a wrapped currency.
    Wrapped,
}

/// The sub-type of permissioned currency.
#[derive(AnchorSerialize, AnchorDeserialize, ToSchema, Clone, PartialEq)]
pub enum PermissionedCurrencyKind {
    /// Represents a non-custodial permissioned currency.
    NonCustodial,
    /// Represents a custodial permissioned currency.
    Custodial,
}
