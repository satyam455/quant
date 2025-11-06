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
