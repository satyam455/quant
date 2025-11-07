use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::CollateralVault;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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
        address = vault.token_account
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let binding = ctx.accounts.user.key();

    let seeds = &[b"vault", binding.as_ref(), &[vault.bump]];
    let signer = &[&seeds[..]]; //@audit

    require!(amount > 0, ErrorCode::InvalidAmount);

    require!(
        vault.available_balance >= amount,
        ErrorCode::InsufficientFunds
    );
    require!(vault.locked_balance == 0, ErrorCode::ActivePosition);

    require!(
        ctx.accounts.user_token_account.owner == ctx.accounts.user.key(),
        ErrorCode::InvalidAuthority
    );

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
        amount,
    )?;

    vault.total_balance = vault
        .total_balance
        .checked_sub(amount)
        .ok_or(ErrorCode::Underflow)?;
    vault.available_balance = vault
        .available_balance
        .checked_sub(amount)
        .ok_or(ErrorCode::Underflow)?;
    vault.total_withdrawn = vault
        .total_withdrawn
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;

    emit!(WithdrawEvent {
        user: ctx.accounts.user.key(),
        amount,
        new_balance: vault.total_balance,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
