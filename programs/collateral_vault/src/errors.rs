use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid mint token")]
    InvalidMint,
    #[msg("Invalid amount provided")]
    InvalidAmount,
    #[msg("Insufficient balance")]
    InsufficientFunds,
    #[msg("Position Active")]
    ActivePosition,
    #[msg("Insufficient Locked Funds")]
    InsufficientLockedFunds,
    #[msg("OverFlow")]
    Overflow,
    #[msg("UnderFlow")]
    Underflow,
}
