use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Could not load price account")]
    PythError,
    #[msg("Failed to serialize price account")]
    TryToSerializePriceAccount,
    #[msg("Price account latest update is too old to safely provide a price")]
    PythPriceTooOld,
    #[msg("Invalid argument provided")]
    InvalidArgument,
    #[msg("One or more token accounts are missing, please add them in remaining accounts")]
    MissingTokenAccounts,
    #[msg("One or more price feeds are missing, please add them in remaining accounts")]
    MissingPriceFeedAccounts,
}
