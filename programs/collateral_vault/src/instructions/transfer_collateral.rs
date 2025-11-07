use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::CollateralVault;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferCollateral<'info> {
    /// Owner of the from_vault (must match from_vault.owner)
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, has_one = owner @ ErrorCode::InvalidAuthority)]
    pub from_vault: Account<'info, CollateralVault>,

    #[account(mut)]
    pub to_vault: Account<'info, CollateralVault>,
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
        from_vault_acc.available_balance >= amount,
        ErrorCode::InsufficientFunds
    );

    // Atomic internal accounting transfer
    from_vault_acc.available_balance = from_vault_acc
        .available_balance
        .checked_sub(amount)
        .ok_or(ErrorCode::Underflow)?;

    to_vault_acc.available_balance = to_vault_acc
        .available_balance
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
