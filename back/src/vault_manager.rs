use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        signature::{Keypair, Signer},
        system_program, sysvar,
        transaction::Transaction,
    },
    Client, Cluster, Program,
};
use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use std::sync::Arc;

pub struct VaultManager {
    pub program: Program<Arc<Keypair>>,
    pub payer: Arc<Keypair>,
    pub usdt_mint: Pubkey,
}

impl VaultManager {
    pub fn new(
        rpc_url: String,
        payer: Keypair,
        program_id: Pubkey,
        usdt_mint: Pubkey,
    ) -> Result<Self> {
        let cluster = Cluster::Custom(rpc_url.clone(), rpc_url);
        let payer = Arc::new(payer);
        let client =
            Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed());
        let program = client.program(program_id)?;
        Ok(Self {
            program,
            payer,
            usdt_mint,
        })
    }

    pub async fn initialize_vault(&self, user: Pubkey) -> Result<String> {
        println!("🚀 Initializing vault for user {}", user);

        let (vault_pda, _) =
            Pubkey::find_program_address(&[b"vault", user.as_ref()], &self.program.id());

        let (vault_token_account, _) =
            Pubkey::find_program_address(&[b"vault_token", user.as_ref()], &self.program.id());

        let (vault_authority, _) =
            Pubkey::find_program_address(&[b"vault_authority"], &self.program.id());

        let ix_data =
            anchor_lang::InstructionData::data(&collateral_vault::instruction::InitializeVault {});

        let accounts = collateral_vault::accounts::InitializeVault {
            user,
            vault: vault_pda,
            usdt_mint: self.usdt_mint,
            vault_token_account,
            vault_authority,
            system_program: system_program::ID,
            token_program: anchor_spl::token::ID,
            rent: sysvar::rent::ID,
        };

        let account_metas = anchor_lang::ToAccountMetas::to_account_metas(&accounts, None);

        let instruction = anchor_lang::solana_program::instruction::Instruction {
            program_id: self.program.id(),
            accounts: account_metas,
            data: ix_data,
        };

        let sig = self.program.rpc().send_and_confirm_transaction(
            &Transaction::new_signed_with_payer(
                &[instruction],
                Some(&self.payer.pubkey()),
                &[&*self.payer],
                self.program.rpc().get_latest_blockhash()?,
            ),
        )?;

        println!("✅ Vault initialized: {}", sig);
        Ok(sig.to_string())
    }

    pub async fn deposit(&self, user: Pubkey, amount: u64) -> Result<String> {
        println!("💰 Depositing {} tokens for {}", amount, user);

        let (vault_pda, _) =
            Pubkey::find_program_address(&[b"vault", user.as_ref()], &self.program.id());

        let (vault_token_account, _) =
            Pubkey::find_program_address(&[b"vault_token", user.as_ref()], &self.program.id());

        let user_token_account =
            anchor_spl::associated_token::get_associated_token_address(&user, &self.usdt_mint);

        let ix_data =
            anchor_lang::InstructionData::data(&collateral_vault::instruction::Deposit { amount });

        let accounts = collateral_vault::accounts::Deposit {
            user,
            vault: vault_pda,
            user_token_account,
            vault_token_account,
            token_program: anchor_spl::token::ID,
        };

        let account_metas = anchor_lang::ToAccountMetas::to_account_metas(&accounts, None);

        let instruction = anchor_lang::solana_program::instruction::Instruction {
            program_id: self.program.id(),
            accounts: account_metas,
            data: ix_data,
        };

        let sig = self.program.rpc().send_and_confirm_transaction(
            &Transaction::new_signed_with_payer(
                &[instruction],
                Some(&self.payer.pubkey()),
                &[&*self.payer],
                self.program.rpc().get_latest_blockhash()?,
            ),
        )?;

        println!("✅ Deposit successful: {}", sig);
        Ok(sig.to_string())
    }

    pub async fn withdraw(&self, user: Pubkey, amount: u64) -> Result<String> {
        println!("🏦 Withdrawing {} tokens for {}", amount, user);

        let (vault_pda, _) =
            Pubkey::find_program_address(&[b"vault", user.as_ref()], &self.program.id());

        let (vault_token_account, _) =
            Pubkey::find_program_address(&[b"vault_token", user.as_ref()], &self.program.id());

        let user_token_account =
            anchor_spl::associated_token::get_associated_token_address(&user, &self.usdt_mint);

        let ix_data =
            anchor_lang::InstructionData::data(&collateral_vault::instruction::Withdraw { amount });

        let accounts = collateral_vault::accounts::Withdraw {
            user,
            vault: vault_pda,
            vault_token_account,
            user_token_account,
            token_program: anchor_spl::token::ID,
        };

        let account_metas = anchor_lang::ToAccountMetas::to_account_metas(&accounts, None);

        let instruction = anchor_lang::solana_program::instruction::Instruction {
            program_id: self.program.id(),
            accounts: account_metas,
            data: ix_data,
        };

        let sig = self.program.rpc().send_and_confirm_transaction(
            &Transaction::new_signed_with_payer(
                &[instruction],
                Some(&self.payer.pubkey()),
                &[&*self.payer],
                self.program.rpc().get_latest_blockhash()?,
            ),
        )?;

        println!("✅ Withdrawal successful: {}", sig);
        Ok(sig.to_string())
    }

    pub async fn lock(&self, user: Pubkey, amount: u64) -> Result<String> {
        println!("🔒 Locking {} collateral for {}", amount, user);

        let (vault_pda, _) =
            Pubkey::find_program_address(&[b"vault", user.as_ref()], &self.program.id());

        let ix_data =
            anchor_lang::InstructionData::data(&collateral_vault::instruction::LockCollateral {
                amount,
            });

        let accounts = collateral_vault::accounts::LockCollateral {
            user,
            vault: vault_pda,
        };

        let account_metas = anchor_lang::ToAccountMetas::to_account_metas(&accounts, None);

        let instruction = anchor_lang::solana_program::instruction::Instruction {
            program_id: self.program.id(),
            accounts: account_metas,
            data: ix_data,
        };

        let sig = self.program.rpc().send_and_confirm_transaction(
            &Transaction::new_signed_with_payer(
                &[instruction],
                Some(&self.payer.pubkey()),
                &[&*self.payer],
                self.program.rpc().get_latest_blockhash()?,
            ),
        )?;

        println!("✅ Collateral locked: {}", sig);
        Ok(sig.to_string())
    }

    pub async fn unlock(&self, user: Pubkey, amount: u64) -> Result<String> {
        println!("🔓 Unlocking {} collateral for {}", amount, user);

        let (vault_pda, _) =
            Pubkey::find_program_address(&[b"vault", user.as_ref()], &self.program.id());

        let ix_data =
            anchor_lang::InstructionData::data(&collateral_vault::instruction::UnlockCollateral {
                amount,
            });

        let accounts = collateral_vault::accounts::UnlockCollateral {
            user,
            vault: vault_pda,
        };

        let account_metas = anchor_lang::ToAccountMetas::to_account_metas(&accounts, None);

        let instruction = anchor_lang::solana_program::instruction::Instruction {
            program_id: self.program.id(),
            accounts: account_metas,
            data: ix_data,
        };

        let sig = self.program.rpc().send_and_confirm_transaction(
            &Transaction::new_signed_with_payer(
                &[instruction],
                Some(&self.payer.pubkey()),
                &[&*self.payer],
                self.program.rpc().get_latest_blockhash()?,
            ),
        )?;

        println!("✅ Collateral unlocked: {}", sig);
        Ok(sig.to_string())
    }

    pub async fn transfer(&self, from: Pubkey, to: Pubkey, amount: u64) -> Result<String> {
        println!("📤 Transferring {} from {} to {}", amount, from, to);

        let (from_vault, _) =
            Pubkey::find_program_address(&[b"vault", from.as_ref()], &self.program.id());

        let (to_vault, _) =
            Pubkey::find_program_address(&[b"vault", to.as_ref()], &self.program.id());

        let ix_data = anchor_lang::InstructionData::data(
            &collateral_vault::instruction::TransferCollateral {
                from_vault,
                to_vault,
                amount,
            },
        );

        let accounts = collateral_vault::accounts::TransferCollateral {
            authority: self.payer.pubkey(),
            from_vault,
            to_vault,
        };

        let account_metas = anchor_lang::ToAccountMetas::to_account_metas(&accounts, None);

        let instruction = anchor_lang::solana_program::instruction::Instruction {
            program_id: self.program.id(),
            accounts: account_metas,
            data: ix_data,
        };

        let sig = self.program.rpc().send_and_confirm_transaction(
            &Transaction::new_signed_with_payer(
                &[instruction],
                Some(&self.payer.pubkey()),
                &[&*self.payer],
                self.program.rpc().get_latest_blockhash()?,
            ),
        )?;

        println!("✅ Transfer successful: {}", sig);
        Ok(sig.to_string())
    }
}

impl Clone for VaultManager {
    fn clone(&self) -> Self {
        let payer = self.payer.clone();
        let cluster = Cluster::Custom(
            self.program.rpc().url().to_string(),
            self.program.rpc().url().to_string(),
        );
        let client =
            Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed());
        let program = client
            .program(self.program.id())
            .expect("Failed to create Anchor program client");

        Self {
            program,
            payer,
            usdt_mint: self.usdt_mint,
        }
    }
}
