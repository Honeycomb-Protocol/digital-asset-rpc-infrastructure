use account_compression::AccountCompressionInstruction;
use bubblegum::BubblegumInstruction;
use hpl_character_manager::HplCharacterManagerAccount;
use hpl_currency_manager::HplCurrencyManagerAccount;
use hpl_hive_control::HplHiveControlAccount;
use hpl_nectar_missions::HplNectarMissionsAccount;
use hpl_nectar_staking::HplNectarStakingAccount;
use hpl_resource_manager::HplResourceManagerAccount;
use mpl_core_program::MplCoreAccountState;
use noop::NoopInstruction;
use token_account::TokenProgramAccount;
use token_extensions::TokenExtensionsProgramAccount;
use token_metadata::TokenMetadataAccountState;

pub mod account_compression;
pub mod bubblegum;
pub mod hpl_character_manager;
pub mod hpl_currency_manager;
pub mod hpl_hive_control;
pub mod hpl_nectar_missions;
pub mod hpl_nectar_staking;
pub mod hpl_resource_manager;
pub mod mpl_core_program;
pub mod noop;
pub mod token_account;
pub mod token_extensions;
pub mod token_metadata;

// Note: `ProgramParseResult` used to contain the following variants that have been deprecated and
// removed from blockbuster since the `version-1.16` tag:
// CandyGuard(&'a CandyGuardAccountData),
// CandyMachine(&'a CandyMachineAccountData),
// CandyMachineCore(&'a CandyMachineCoreAccountData),
//
// Candy Machine V3 parsing was removed because Candy Guard (`mpl-candy-guard`) and
// Candy Machine Core (`mpl-candy-machine-core`) were dependent upon a specific Solana
// version (1.16), there was no Candy Machine parsing in DAS (`digital-asset-rpc-infrastructure`),
// and we wanted to use the Rust clients for Bubblegum and Token Metadata so that going forward we
// could more easily update blockbuster to new Solana versions.
//
// Candy Machine V2 (`mpl-candy-machine`) parsing was removed at the same time as V3 because even
// though it did not depend on the `mpl-candy-machine` crate, it was also not being used by DAS.
pub enum ProgramParseResult<'a> {
    Unknown,
    Bubblegum(&'a BubblegumInstruction),
    MplCore(&'a MplCoreAccountState),
    TokenMetadata(&'a TokenMetadataAccountState),
    TokenProgramAccount(&'a TokenProgramAccount),
    TokenExtensionsProgramAccount(&'a TokenExtensionsProgramAccount),
    AccountCompression(&'a AccountCompressionInstruction),
    Noop(&'a NoopInstruction),
    HplHiveControl(&'a HplHiveControlAccount),
    HplCharacterManager(&'a HplCharacterManagerAccount),
    HplCurrencyManager(&'a HplCurrencyManagerAccount),
    HplResourceManager(&'a HplResourceManagerAccount),
    HplNectarMissions(&'a HplNectarMissionsAccount),
    HplNectarStaking(&'a HplNectarStakingAccount),
}
