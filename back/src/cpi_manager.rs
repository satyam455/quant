use crate::vault_manager::VaultManager;
use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use std::sync::Arc;

pub struct CPIManager {
    vault_manager: Arc<VaultManager>,
}

impl CPIManager {
    pub fn new(vault_manager: Arc<VaultManager>) -> Self {
        Self { vault_manager }
    }

    /// Lock collateral for a position (called by position manager)
    pub async fn lock_for_position(
        &self,
        user: Pubkey,
        amount: u64,
        position_id: String,
    ) -> Result<String> {
        println!("🔐 Locking {} tokens for position {}", amount, position_id);

        // Call the lock instruction
        let sig = self.vault_manager.lock(user, amount).await?;

        println!("✅ Locked successfully: {}", sig);
        Ok(sig)
    }

    /// Unlock collateral after position close
    pub async fn unlock_after_close(
        &self,
        user: Pubkey,
        amount: u64,
        position_id: String,
    ) -> Result<String> {
        println!(
            "🔓 Unlocking {} tokens after position {} close",
            amount, position_id
        );

        let sig = self.vault_manager.unlock(user, amount).await?;

        println!("✅ Unlocked successfully: {}", sig);
        Ok(sig)
    }

    /// Transfer collateral for liquidation
    pub async fn transfer_for_liquidation(
        &self,
        from: Pubkey,
        to: Pubkey,
        amount: u64,
        liquidation_id: String,
    ) -> Result<String> {
        println!(
            "⚡ Liquidation transfer: {} tokens from {} to {} (ID: {})",
            amount, from, to, liquidation_id
        );

        let sig = self.vault_manager.transfer(from, to, amount).await?;

        println!("✅ Liquidation transfer successful: {}", sig);
        Ok(sig)
    }

    /// Batch lock for multiple positions
    pub async fn batch_lock(&self, operations: Vec<(Pubkey, u64, String)>) -> Result<Vec<String>> {
        let mut signatures = Vec::new();

        for (user, amount, position_id) in operations {
            match self.lock_for_position(user, amount, position_id).await {
                Ok(sig) => signatures.push(sig),
                Err(e) => {
                    eprintln!("❌ Failed to lock for user {}: {}", user, e);
                    return Err(e);
                }
            }
        }

        Ok(signatures)
    }

    /// Handle CPI errors gracefully
    pub async fn safe_lock(&self, user: Pubkey, amount: u64) -> Result<Option<String>> {
        match self.vault_manager.lock(user, amount).await {
            Ok(sig) => Ok(Some(sig)),
            Err(e) => {
                eprintln!("⚠️  Lock failed for {}: {}", user, e);
                // Log error but don't fail the entire operation
                Ok(None)
            }
        }
    }
}
