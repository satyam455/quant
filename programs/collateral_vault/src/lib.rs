use anchor_lang::prelude::*;
// use anchor_spl::token::{Token, TokenAccount};

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("4NYik8PfZkQdj89AjVxX8LWHZyPKWQ647XKNpCMk6gAR");

#[program]
pub mod collateral_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        instructions::initialize_vault::handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::withdraw(ctx, amount)
    }

    pub fn lock_collateral(ctx: Context<LockCollateral>, amount: u64) -> Result<()> {
        instructions::lock::lock_collateral(ctx, amount)
    }

    pub fn unlock_collateral(ctx: Context<UnlockCollateral>, amount: u64) -> Result<()> {
        instructions::unlock::unlock_collateral(ctx, amount)
    }

    pub fn transfer_collateral(
        ctx: Context<TransferCollateral>,
        from_vault: Pubkey,
        to_vault: Pubkey,
        amount: u64,
    ) -> Result<()> {
        instructions::transfer_collateral::transfer_collateral(ctx, from_vault, to_vault, amount)
    }
}
