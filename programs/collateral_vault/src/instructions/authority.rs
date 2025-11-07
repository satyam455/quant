use crate::errors::ErrorCode;
use crate::events::{AuthorizedProgramAdded, AuthorizedProgramRemoved};
use crate::state::{CollateralVault, VaultAuthority};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddAuthorizedProgram<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        constraint = vault.vault_authority == vault_authority.key() @ ErrorCode::InvalidVaultAuthority
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        seeds = [b"vault_authority", vault.key().as_ref()],
        bump = vault_authority.bump,
        has_one = vault @ ErrorCode::InvalidVaultAuthority
    )]
    pub vault_authority: Account<'info, VaultAuthority>,
}

pub fn add_authorized_program(ctx: Context<AddAuthorizedProgram>, program: Pubkey) -> Result<()> {
    require!(program != Pubkey::default(), ErrorCode::InvalidAuthority);

    let authority = &mut ctx.accounts.vault_authority;
    require!(
        !authority.authorized_programs.contains(&program),
        ErrorCode::AuthorizationAlreadyExists
    );
    require!(
        authority.authorized_programs.len() < VaultAuthority::MAX_AUTHORIZED_PROGRAMS,
        ErrorCode::AuthorizedProgramsCapacity
    );

    authority.authorized_programs.push(program);

    emit!(AuthorizedProgramAdded {
        vault: ctx.accounts.vault.key(),
        program,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RemoveAuthorizedProgram<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
        constraint = vault.vault_authority == vault_authority.key() @ ErrorCode::InvalidVaultAuthority
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        seeds = [b"vault_authority", vault.key().as_ref()],
        bump = vault_authority.bump,
        has_one = vault @ ErrorCode::InvalidVaultAuthority
    )]
    pub vault_authority: Account<'info, VaultAuthority>,
}

pub fn remove_authorized_program(
    ctx: Context<RemoveAuthorizedProgram>,
    program: Pubkey,
) -> Result<()> {
    let authority = &mut ctx.accounts.vault_authority;
    if let Some(index) = authority
        .authorized_programs
        .iter()
        .position(|p| p == &program)
    {
        authority.authorized_programs.swap_remove(index);

        emit!(AuthorizedProgramRemoved {
            vault: ctx.accounts.vault.key(),
            program,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    } else {
        err!(ErrorCode::Unauthorized)
    }
}
