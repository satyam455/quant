use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_lang::AccountDeserialize;
use anyhow::Result;
use collateral_vault::state::CollateralVault;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::{BalanceSnapshot, Database};
use crate::vault_manager::VaultManager;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultBalance {
    pub owner: String,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceAlert {
    pub vault: String,
    pub alert_type: AlertType,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    LowBalance,
    HighLockedRatio,
    UnauthorizedAccess,
    Discrepancy,
}

pub struct BalanceTracker {
    vault_manager: Arc<VaultManager>,
    database: Arc<Database>,
    cached_balances: Arc<RwLock<std::collections::HashMap<String, VaultBalance>>>,
    alerts: Arc<RwLock<Vec<BalanceAlert>>>,
}

impl BalanceTracker {
    pub fn new(vault_manager: Arc<VaultManager>, database: Arc<Database>) -> Self {
        Self {
            vault_manager,
            database,
            cached_balances: Arc::new(RwLock::new(std::collections::HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Fetch vault balance from on-chain
    pub async fn get_vault_balance(&self, user: Pubkey) -> Result<VaultBalance> {
        // Derive vault PDA
        let (vault_pda, _) = Pubkey::find_program_address(
            &[b"vault", user.as_ref()],
            &self.vault_manager.program.id(),
        );

        // Fetch account data
        let account = self.vault_manager.program.rpc().get_account(&vault_pda)?;

        // Deserialize vault data
        let mut data: &[u8] = &account.data;
        let vault = CollateralVault::try_deserialize(&mut data)?;

        let balance = VaultBalance {
            owner: vault.owner.to_string(),
            total_balance: vault.total_balance,
            locked_balance: vault.locked_balance,
            available_balance: vault.available_balance,
            total_deposited: vault.total_deposited,
            total_withdrawn: vault.total_withdrawn,
            created_at: vault.created_at,
        };

        // Update cache
        let mut cache = self.cached_balances.write().await;
        cache.insert(user.to_string(), balance.clone());
        drop(cache);

        // Persist snapshot + balances
        let snapshot = BalanceSnapshot {
            id: Uuid::new_v4().to_string(),
            user: balance.owner.clone(),
            total_balance: balance.total_balance,
            locked_balance: balance.locked_balance,
            available_balance: balance.available_balance,
            timestamp: Utc::now().timestamp(),
        };
        self.database.insert_snapshot(snapshot).await?;
        self.database
            .update_vault_balances(
                &balance.owner,
                balance.total_balance,
                balance.locked_balance,
                balance.available_balance,
            )
            .await?;

        // Check for alerts
        self.check_balance_alerts(&balance).await;

        Ok(balance)
    }

    /// Get cached balance (fast, may be stale)
    pub async fn get_cached_balance(&self, user: &str) -> Option<VaultBalance> {
        let cache = self.cached_balances.read().await;
        cache.get(user).cloned()
    }

    /// Calculate total value locked across all vaults
    pub async fn calculate_tvl(&self) -> u64 {
        let cache = self.cached_balances.read().await;
        cache.values().map(|v| v.total_balance).sum()
    }

    /// Monitor and check for balance alerts
    async fn check_balance_alerts(&self, balance: &VaultBalance) {
        let mut alerts = self.alerts.write().await;

        // Alert 1: Low balance (less than 1000 tokens)
        if balance.available_balance < 1_000_000 && balance.available_balance > 0 {
            let timestamp = Utc::now().timestamp();
            let alert = BalanceAlert {
                vault: balance.owner.clone(),
                alert_type: AlertType::LowBalance,
                message: format!(
                    "Low available balance: {} tokens",
                    balance.available_balance
                ),
                timestamp,
            };
            alerts.push(alert.clone());
            let _ = self
                .database
                .create_alert(
                    "LOW_BALANCE",
                    "MEDIUM",
                    Some(balance.owner.as_str()),
                    &alert.message,
                )
                .await;
        }

        // Alert 2: High locked ratio (>80% locked)
        if balance.total_balance > 0 {
            let locked_ratio =
                (balance.locked_balance as f64 / balance.total_balance as f64) * 100.0;
            if locked_ratio > 80.0 {
                let message = format!("High locked ratio: {:.2}%", locked_ratio);
                let timestamp = Utc::now().timestamp();
                let alert = BalanceAlert {
                    vault: balance.owner.clone(),
                    alert_type: AlertType::HighLockedRatio,
                    message: message.clone(),
                    timestamp,
                };
                alerts.push(alert.clone());
                let _ = self
                    .database
                    .create_alert(
                        "HIGH_LOCKED_RATIO",
                        "HIGH",
                        Some(balance.owner.as_str()),
                        &message,
                    )
                    .await;
            }
        }

        // Keep only last 100 alerts
        let len = alerts.len();
        if len > 100 {
            alerts.drain(0..(len - 100));
        }
    }

    /// Get all alerts
    pub async fn get_alerts(&self) -> Vec<BalanceAlert> {
        if let Ok(active_alerts) = self.database.get_active_alerts().await {
            if !active_alerts.is_empty() {
                return active_alerts
                    .into_iter()
                    .map(|alert| BalanceAlert {
                        vault: alert.user.unwrap_or_default(),
                        alert_type: match alert.alert_type.as_str() {
                            "LOW_BALANCE" => AlertType::LowBalance,
                            "HIGH_LOCKED_RATIO" => AlertType::HighLockedRatio,
                            "BALANCE_DISCREPANCY" => AlertType::Discrepancy,
                            _ => AlertType::UnauthorizedAccess,
                        },
                        message: alert.message,
                        timestamp: alert.timestamp,
                    })
                    .collect();
            }
        }

        let alerts = self.alerts.read().await;
        alerts.clone()
    }

    /// Reconcile on-chain vs cached state
    pub async fn reconcile(&self, user: Pubkey) -> Result<bool> {
        let cached = self.get_cached_balance(&user.to_string()).await;
        let on_chain = self.get_vault_balance(user).await?;

        if let Some(cached_balance) = cached {
            // Check for discrepancies
            if cached_balance.total_balance != on_chain.total_balance {
                let mut alerts = self.alerts.write().await;
                let user_str = user.to_string();
                let message = format!(
                    "Balance mismatch! Cached: {}, On-chain: {}",
                    cached_balance.total_balance, on_chain.total_balance
                );
                let timestamp = Utc::now().timestamp();
                alerts.push(BalanceAlert {
                    vault: user_str.clone(),
                    alert_type: AlertType::Discrepancy,
                    message: message.clone(),
                    timestamp,
                });
                let _ = self
                    .database
                    .create_alert(
                        "BALANCE_DISCREPANCY",
                        "CRITICAL",
                        Some(user_str.as_str()),
                        &message,
                    )
                    .await;
                let _ = self
                    .database
                    .log_reconciliation(
                        user_str.as_str(),
                        on_chain.total_balance,
                        cached_balance.total_balance,
                        "MISMATCH",
                    )
                    .await;
                return Ok(false);
            }
        }

        let user_str = user.to_string();
        let _ = self
            .database
            .log_reconciliation(
                user_str.as_str(),
                on_chain.total_balance,
                on_chain.total_balance,
                "MATCH",
            )
            .await;

        Ok(true)
    }

    /// Start background monitoring task
    pub fn start_monitoring(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                // Monitor TVL
                let tvl = self.calculate_tvl().await;
                println!("📊 Current TVL: {} tokens", tvl);

                // You can add more monitoring logic here
            }
        });
    }
}
