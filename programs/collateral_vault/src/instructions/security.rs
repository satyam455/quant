use crate::errors::ErrorCode;
use crate::events::{WithdrawalExecuted, WithdrawalRequested};
use crate::state::{CollateralVault, WithdrawalRequest};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

pub const WITHDRAWAL_DELAY_SECONDS: i64 = 86_400; // 24 hours

#[derive(Accounts)]
#[instruction(request_id: u64)]
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
        seeds = [b"withdrawal", vault.key().as_ref(), &request_id.to_le_bytes()],
        bump,
    )]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,

    pub system_program: Program<'info, System>,
}

pub fn request_withdrawal(
    ctx: Context<RequestWithdrawal>,
    request_id: u64,
    amount: u64,
) -> Result<()> {
    let vault = &ctx.accounts.vault;
    let request = &mut ctx.accounts.withdrawal_request;

    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(
        vault.available_balance >= amount,
        ErrorCode::InsufficientFunds
    );
    if vault.locked_balance > 0 {
        return err!(ErrorCode::ActivePosition);
    }

    let current_time = Clock::get()?.unix_timestamp;

    request.vault = vault.key();
    request.user = ctx.accounts.user.key();
    request.amount = amount;
    request.requested_at = current_time;
    request.available_at = current_time + WITHDRAWAL_DELAY_SECONDS;
    request.request_id = request_id;
    request.executed = false;
    request.bump = ctx.bumps.withdrawal_request;

    emit!(WithdrawalRequested {
        user: ctx.accounts.user.key(),
        vault: vault.key(),
        amount,
        available_at: request.available_at,
        timestamp: current_time,
    });

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
        constraint = vault.vault_authority != Pubkey::default() @ ErrorCode::InvalidVaultAuthority
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        close = user,
        seeds = [b"withdrawal", vault.key().as_ref(), &withdrawal_request.request_id.to_le_bytes()],
        bump = withdrawal_request.bump,
        constraint = !withdrawal_request.executed @ ErrorCode::AlreadyExecuted,
        constraint = withdrawal_request.user == user.key() @ ErrorCode::Unauthorized,
        constraint = withdrawal_request.vault == vault.key() @ ErrorCode::InvalidWithdrawalRequest
    )]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,

    #[account(
        mut,
        address = vault.token_account
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

pub fn execute_withdrawal(ctx: Context<ExecuteWithdrawal>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let request = &mut ctx.accounts.withdrawal_request;
    let vault = &mut ctx.accounts.vault;

    require!(
        current_time >= request.available_at,
        ErrorCode::WithdrawalDelayNotMet
    );
    require!(
        vault.available_balance >= request.amount,
        ErrorCode::InsufficientFunds
    );
    require!(vault.locked_balance == 0, ErrorCode::ActivePosition);
    require!(
        ctx.accounts.user_token_account.owner == ctx.accounts.user.key(),
        ErrorCode::InvalidAuthority
    );

    let binding = ctx.accounts.user.key();
    let seeds = &[b"vault", binding.as_ref(), &[vault.bump]];
    let signer = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: vault.to_account_info(),
            },
            signer,
        ),
        request.amount,
    )?;

    vault.total_balance = vault
        .total_balance
        .checked_sub(request.amount)
        .ok_or(ErrorCode::Underflow)?;
    vault.available_balance = vault
        .available_balance
        .checked_sub(request.amount)
        .ok_or(ErrorCode::Underflow)?;
    vault.total_withdrawn = vault
        .total_withdrawn
        .checked_add(request.amount)
        .ok_or(ErrorCode::Overflow)?;

    request.executed = true;

    emit!(WithdrawalExecuted {
        user: ctx.accounts.user.key(),
        vault: vault.key(),
        amount: request.amount,
        timestamp: current_time,
    });

    Ok(())
}
