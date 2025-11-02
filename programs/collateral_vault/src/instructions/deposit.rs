use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::CollateralVault;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
#[instruction()]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    // Transfer USDT from user to vault using Cross-Program Invocation (CPI)
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update vault state
    let vault = &mut ctx.accounts.vault;
    vault.total_balance += amount;
    vault.available_balance += amount;
    vault.total_deposited += amount;

    // Emit event for off-chain indexing
    emit!(DepositEvent {
        user: ctx.accounts.user.key(),
        amount,
        new_balance: vault.total_balance,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
