use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::{CollateralVault, VaultAuthority};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use std::collections::HashSet;

#[derive(Accounts)]
#[instruction(authorized_programs: Vec<Pubkey>)]
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
        init,
        payer = user,
        space = 8 + VaultAuthority::MAX_SIZE,
        seeds = [b"vault_authority", vault.key().as_ref()],
        bump
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

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

pub fn handler(ctx: Context<InitializeVault>, authorized_programs: Vec<Pubkey>) -> Result<()> {
    require!(
        authorized_programs.len() <= VaultAuthority::MAX_AUTHORIZED_PROGRAMS,
        ErrorCode::AuthorizedProgramsCapacity
    );

    let mut deduped = Vec::with_capacity(authorized_programs.len());
    let mut seen = HashSet::with_capacity(authorized_programs.len());
    for program in authorized_programs {
        require!(program != Pubkey::default(), ErrorCode::InvalidAuthority);
        require!(seen.insert(program), ErrorCode::AuthorizationAlreadyExists);
        deduped.push(program);
    }

    let vault = &mut ctx.accounts.vault;

    vault.owner = ctx.accounts.user.key();
    vault.token_account = ctx.accounts.vault_token_account.key();
    vault.vault_authority = ctx.accounts.vault_authority.key();
    vault.total_balance = 0;
    vault.locked_balance = 0;
    vault.available_balance = 0;
    vault.total_deposited = 0;
    vault.total_withdrawn = 0;
    vault.created_at = Clock::get()?.unix_timestamp;
    vault.bump = ctx.bumps.vault;

    let vault_authority = &mut ctx.accounts.vault_authority;
    vault_authority.vault = vault.key();
    vault_authority.authorized_programs = deduped;
    vault_authority.bump = ctx.bumps.vault_authority;

    emit!(VaultInitialized {
        user: ctx.accounts.user.key(),
        vault: ctx.accounts.vault.key(),
        amount: 0,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
