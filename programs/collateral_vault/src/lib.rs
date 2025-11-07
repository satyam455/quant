use anchor_lang::prelude::*;
// use anchor_spl::token::{Token, TokenAccount};

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("GfHdK9T6kBwS55D9pv97CbNE9PdP4kpASxMipM7gWSKa");

#[program]
pub mod collateral_vault {
    use super::*;

    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        authorized_programs: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_vault::handler(ctx, authorized_programs)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::withdraw(ctx, amount)
    }

    pub fn lock_collateral(ctx: Context<LockCollateral>, amount: u64) -> Result<()> {
        instructions::lock::lock_collateral(ctx, amount)
    }

    pub fn unlock_collateral(ctx: Context<UnlockCollateral>, amount: u64) -> Result<()> {
        instructions::unlock::unlock_collateral(ctx, amount)
    }

    pub fn transfer_collateral(
        ctx: Context<TransferCollateral>,
        from_vault: Pubkey,
        to_vault: Pubkey,
        amount: u64,
    ) -> Result<()> {
        instructions::transfer_collateral::transfer_collateral(ctx, from_vault, to_vault, amount)
    }

    pub fn add_authorized_program(
        ctx: Context<AddAuthorizedProgram>,
        program: Pubkey,
    ) -> Result<()> {
        instructions::authority::add_authorized_program(ctx, program)
    }

    pub fn remove_authorized_program(
        ctx: Context<RemoveAuthorizedProgram>,
        program: Pubkey,
    ) -> Result<()> {
        instructions::authority::remove_authorized_program(ctx, program)
    }

    pub fn request_withdrawal(
        ctx: Context<RequestWithdrawal>,
        request_id: u64,
        amount: u64,
    ) -> Result<()> {
        instructions::security::request_withdrawal(ctx, request_id, amount)
    }

    pub fn execute_withdrawal(ctx: Context<ExecuteWithdrawal>) -> Result<()> {
        instructions::security::execute_withdrawal(ctx)
    }

    pub fn initialize_multisig(
        ctx: Context<InitializeMultisig>,
        signers: Vec<Pubkey>,
        threshold: u8,
    ) -> Result<()> {
        instructions::multisig::initialize_multisig(ctx, signers, threshold)
    }
}
