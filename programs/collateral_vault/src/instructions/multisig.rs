use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::{CollateralVault, MultisigConfig};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeMultisig<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<MultisigConfig>(),
        seeds = [b"multisig", vault.key().as_ref()],
        bump,
    )]
    pub multisig_config: Account<'info, MultisigConfig>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_multisig(
    ctx: Context<InitializeMultisig>,
    signers: Vec<Pubkey>,
    threshold: u8,
) -> Result<()> {
    require!(threshold > 0, ErrorCode::InvalidAmount);
    require!(
        threshold as usize <= signers.len(),
        ErrorCode::InvalidAmount
    );

    let multisig = &mut ctx.accounts.multisig_config;
    multisig.vault = ctx.accounts.vault.key();
    multisig.signers = signers;
    multisig.threshold = threshold;
    multisig.bump = ctx.bumps.multisig_config;

    emit!(MultisigInitialized {
        vault: ctx.accounts.vault.key(),
        threshold,
        signers: multisig.signers.len() as u8,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
