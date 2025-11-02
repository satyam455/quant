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

    #[account(mut)]
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

    vault.total_balance -= amount; //@note use secure under achor only to avoid underflow
    vault.available_balance -= amount;
    vault.total_withdrawn += amount;

    emit!(WithdrawEvent {
        user: ctx.accounts.user.key(),
        amount,
        new_balance: vault.total_balance,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
