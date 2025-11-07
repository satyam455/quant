use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid mint token")]
    InvalidMint,
    #[msg("Invalid amount provided")]
    InvalidAmount,
    #[msg("Duplicate authorization entry")]
    AuthorizationAlreadyExists,
    #[msg("Maximum number of authorized programs reached")]
    AuthorizedProgramsCapacity,
    #[msg("Program not authorized")]
    Unauthorized,
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
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Withdrawal request not yet available")]
    WithdrawalDelayNotMet,
    #[msg("Withdrawal request already executed")]
    AlreadyExecuted,
    #[msg("Vault authority mismatch")]
    InvalidVaultAuthority,
    #[msg("Withdrawal request does not match vault")]
    InvalidWithdrawalRequest,
}
