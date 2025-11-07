use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::{CollateralVault, VaultAuthority};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferCollateral<'info> {
    #[account(mut)]
    pub operator: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", from_vault.owner.as_ref()],
        bump = from_vault.bump,
        constraint = from_vault.vault_authority == from_vault_authority.key() @ ErrorCode::InvalidVaultAuthority
    )]
    pub from_vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        seeds = [b"vault_authority", from_vault.key().as_ref()],
        bump = from_vault_authority.bump,
        constraint = from_vault_authority.vault == from_vault.key() @ ErrorCode::InvalidVaultAuthority
    )]
    pub from_vault_authority: Account<'info, VaultAuthority>,

    #[account(
        mut,
        seeds = [b"vault", to_vault.owner.as_ref()],
        bump = to_vault.bump,
        constraint = to_vault.vault_authority == to_vault_authority.key() @ ErrorCode::InvalidVaultAuthority
    )]
    pub to_vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        seeds = [b"vault_authority", to_vault.key().as_ref()],
        bump = to_vault_authority.bump,
        constraint = to_vault_authority.vault == to_vault.key() @ ErrorCode::InvalidVaultAuthority
    )]
    pub to_vault_authority: Account<'info, VaultAuthority>,

    /// CHECK: Validated as executable + authorized in handler
    pub authority_program: AccountInfo<'info>,
}

pub fn transfer_collateral(
    ctx: Context<TransferCollateral>,
    from_vault: Pubkey,
    to_vault: Pubkey,
    amount: u64,
) -> Result<()> {
    let from_vault_acc = &mut ctx.accounts.from_vault;
    let to_vault_acc = &mut ctx.accounts.to_vault;

    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(
        ctx.accounts.operator.key() == from_vault_acc.owner
            || ctx.accounts.operator.key() == to_vault_acc.owner,
        ErrorCode::InvalidAuthority
    );
    require!(
        from_vault_acc.key() == from_vault && to_vault_acc.key() == to_vault,
        ErrorCode::InvalidAuthority
    );
    require!(
        ctx.accounts
            .from_vault_authority
            .authorized_programs
            .iter()
            .any(|program| program == ctx.accounts.authority_program.key),
        ErrorCode::Unauthorized
    );
    require!(
        ctx.accounts
            .to_vault_authority
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
        from_vault_acc.available_balance >= amount,
        ErrorCode::InsufficientFunds
    );

    // Atomic internal accounting transfer
    from_vault_acc.available_balance = from_vault_acc
        .available_balance
        .checked_sub(amount)
        .ok_or(ErrorCode::Underflow)?;
    from_vault_acc.total_balance = from_vault_acc
        .total_balance
        .checked_sub(amount)
        .ok_or(ErrorCode::Underflow)?;

    to_vault_acc.available_balance = to_vault_acc
        .available_balance
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;
    to_vault_acc.total_balance = to_vault_acc
        .total_balance
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;

    emit!(CollateralTransferred {
        from_vault,
        to_vault,
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
