use anchor_lang::prelude::*;

#[account]
pub struct CollateralVault {
    pub owner: Pubkey,
    pub token_account: Pubkey,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
pub struct MultisigConfig {
    pub vault: Pubkey,
    pub signers: Vec<Pubkey>,
    pub threshold: u8,
    pub bump: u8,
}

#[account]
pub struct WithdrawalRequest {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub requested_at: i64,
    pub available_at: i64,
    pub executed: bool,
    pub bump: u8,
}

#[account]
pub struct WithdrawalWhitelist {
    pub vault: Pubkey,
    pub addresses: Vec<Pubkey>,
    pub bump: u8,
}
