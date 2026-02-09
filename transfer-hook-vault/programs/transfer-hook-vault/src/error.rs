use anchor_lang::prelude::*;

#[error_code]
pub enum MyError {
    #[msg("Invalid account size")]
    InvalidAccountSize,
    #[msg("Mint initialization failed")]
    MintInitializationFailed,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Recipient is Unauthorized")]
    ResUnauthorized,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Arithmetic overflow")]
    Overflow,
}