use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::CollateralVault;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
#[instruction()]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<CollateralVault>(),
        seeds = [b"vault", user.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        constraint = usdt_mint.decimals == 6 @ ErrorCode::InvalidMint
    )]
    pub usdt_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        token::mint = usdt_mint,
        token::authority = vault,
        seeds = [b"vault_token", user.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeVault>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    vault.owner = ctx.accounts.user.key();
    vault.token_account = ctx.accounts.vault_token_account.key();
    vault.total_balance = 0;
    vault.locked_balance = 0;
    vault.available_balance = 0;
    vault.total_deposited = 0;
    vault.total_withdrawn = 0;
    vault.created_at = Clock::get()?.unix_timestamp;
    vault.bump = ctx.bumps.vault;

    emit!(VaultInitialized {
        user: ctx.accounts.user.key(),
        vault: ctx.accounts.vault.key(),
        amount: 0,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
