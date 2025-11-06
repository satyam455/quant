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
    pub authority: Pubkey,
}
