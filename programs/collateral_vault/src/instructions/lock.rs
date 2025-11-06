use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::CollateralVault;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct LockCollateral<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,
}

pub fn lock_collateral(ctx: Context<LockCollateral>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(
        vault.available_balance >= amount,
        ErrorCode::InsufficientFunds
    );

    // Move funds from available → locked
    vault.available_balance -= amount;
    vault.locked_balance += amount;

    emit!(CollateralLocked {
        user: ctx.accounts.user.key(),
        amount,
        new_locked_balance: vault.locked_balance,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
