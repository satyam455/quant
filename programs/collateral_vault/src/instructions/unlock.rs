use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::{CollateralVault, VaultAuthority};
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

    #[account(
        mut,
        seeds = [b"vault_authority", vault.key().as_ref()],
        bump = vault_authority.bump,
        has_one = vault @ ErrorCode::InvalidVaultAuthority,
        constraint = vault.vault_authority == vault_authority.key() @ ErrorCode::InvalidVaultAuthority
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    /// CHECK: CPI caller; validated in handler
    pub authority_program: AccountInfo<'info>,
}

pub fn unlock_collateral(ctx: Context<UnlockCollateral>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(
        ctx.accounts
            .vault_authority
            .authorized_programs
            .iter()
            .any(|program| program == ctx.accounts.authority_program.key),
        ErrorCode::Unauthorized
    );
    require!(
        ctx.accounts.authority_program.executable,
        ErrorCode::Unauthorized
    );
    require!(
        vault.locked_balance >= amount,
        ErrorCode::InsufficientLockedFunds
    );

    // Move funds from locked → available
    vault.locked_balance = vault
        .locked_balance
        .checked_sub(amount)
        .ok_or(ErrorCode::Underflow)?;
    vault.available_balance = vault
        .available_balance
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;

    emit!(CollateralUnlocked {
        user: ctx.accounts.user.key(),
        amount,
        new_available_balance: vault.available_balance,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
