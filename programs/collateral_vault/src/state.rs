use anchor_lang::prelude::*;

#[account]
pub struct CollateralVault {
    pub owner: Pubkey,
    pub token_account: Pubkey,
    pub vault_authority: Pubkey,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
pub struct VaultAuthority {
    pub vault: Pubkey,
    pub authorized_programs: Vec<Pubkey>,
    pub bump: u8,
}

impl VaultAuthority {
    pub const MAX_AUTHORIZED_PROGRAMS: usize = 16;
    pub const MAX_SIZE: usize = 32 + 4 + (Self::MAX_AUTHORIZED_PROGRAMS * 32) + 1;
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
    pub request_id: u64,
    pub executed: bool,
    pub bump: u8,
}

#[account]
pub struct WithdrawalWhitelist {
    pub vault: Pubkey,
    pub addresses: Vec<Pubkey>,
    pub bump: u8,
}
