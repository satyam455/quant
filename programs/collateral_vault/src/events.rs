use anchor_lang::prelude::*;

#[event]
pub struct VaultInitialized {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct CollateralLocked {
    pub user: Pubkey,
    pub amount: u64,
    pub new_locked_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct CollateralUnlocked {
    pub user: Pubkey,
    pub amount: u64,
    pub new_available_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct CollateralTransferred {
    pub from_vault: Pubkey,
    pub to_vault: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct AuthorizedProgramAdded {
    pub vault: Pubkey,
    pub program: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AuthorizedProgramRemoved {
    pub vault: Pubkey,
    pub program: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalRequested {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub available_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalExecuted {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct MultisigInitialized {
    pub vault: Pubkey,
    pub threshold: u8,
    pub signer_count: u8,
    pub timestamp: i64,
}
