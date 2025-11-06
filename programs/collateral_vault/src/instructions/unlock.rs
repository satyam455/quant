use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::CollateralVault;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UnlockCollateral<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,
}

pub fn unlock_collateral(ctx: Context<UnlockCollateral>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(
        vault.locked_balance >= amount,
        ErrorCode::InsufficientLockedFunds
    );

    // Move funds from locked → available
    vault.locked_balance -= amount;
    vault.available_balance += amount;

    emit!(CollateralUnlocked {
        user: ctx.accounts.user.key(),
        amount,
        new_available_balance: vault.available_balance,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
