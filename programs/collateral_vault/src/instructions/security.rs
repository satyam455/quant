use crate::errors::ErrorCode;
use crate::state::{CollateralVault, WithdrawalRequest};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RequestWithdrawal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<WithdrawalRequest>(),
        seeds = [b"withdrawal", vault.key().as_ref(), &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump,
    )]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,

    pub system_program: Program<'info, System>,
}

pub fn request_withdrawal(ctx: Context<RequestWithdrawal>, amount: u64) -> Result<()> {
    let vault = &ctx.accounts.vault;
    let request = &mut ctx.accounts.withdrawal_request;

    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(
        vault.available_balance >= amount,
        ErrorCode::InsufficientFunds
    );

    let current_time = Clock::get()?.unix_timestamp;

    request.vault = vault.key();
    request.user = ctx.accounts.user.key();
    request.amount = amount;
    request.requested_at = current_time;
    request.available_at = current_time + 86400; // 24-hour delay
    request.executed = false;
    request.bump = ctx.bumps.withdrawal_request;

    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
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
        seeds = [b"withdrawal", vault.key().as_ref(), &withdrawal_request.requested_at.to_le_bytes()],
        bump = withdrawal_request.bump,
        constraint = !withdrawal_request.executed @ ErrorCode::AlreadyExecuted,
        constraint = withdrawal_request.user == user.key() @ ErrorCode::Unauthorized,
    )]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,
    // ... token accounts
}

pub fn execute_withdrawal(ctx: Context<ExecuteWithdrawal>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let request = &ctx.accounts.withdrawal_request;

    require!(
        current_time >= request.available_at,
        ErrorCode::WithdrawalDelayNotMet
    );

    // Perform actual withdrawal
    // ... (use existing withdraw logic)

    ctx.accounts.withdrawal_request.executed = true;

    Ok(())
}
