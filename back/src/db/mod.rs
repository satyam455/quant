use crate::db::postgres::Alert;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod models;
pub mod postgres;

pub use models::*;
pub use postgres::PostgresDatabase;

#[derive(Clone)]
pub struct Database {
    postgres: Option<PostgresDatabase>,
    transactions: Arc<RwLock<Vec<TransactionRecord>>>,
    balance_snapshots: Arc<RwLock<Vec<BalanceSnapshot>>>,
    audit_logs: Arc<RwLock<Vec<AuditLog>>>,
}

impl Database {
    pub fn new(postgres: Option<PostgresDatabase>) -> Self {
        Self {
            postgres,
            transactions: Arc::new(RwLock::new(Vec::new())),
            balance_snapshots: Arc::new(RwLock::new(Vec::new())),
            audit_logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn has_persistence(&self) -> bool {
        self.postgres.is_some()
    }

    pub async fn register_vault(
        &self,
        owner: &Pubkey,
        vault_pda: &Pubkey,
        token_account: &Pubkey,
    ) -> Result<()> {
        if let Some(pg) = &self.postgres {
            pg.create_vault(
                &owner.to_string(),
                &vault_pda.to_string(),
                &token_account.to_string(),
            )
            .await
            .context("failed to persist vault metadata")?;
        }
        Ok(())
    }

    pub async fn update_vault_balances(
        &self,
        owner: &str,
        total: u64,
        locked: u64,
        available: u64,
    ) -> Result<()> {
        if let Some(pg) = &self.postgres {
            if let Some(vault) = pg
                .get_vault_by_owner(owner)
                .await
                .context("failed to fetch vault for balance update")?
            {
                pg.update_vault_balance(vault.id, total as i64, locked as i64, available as i64)
                    .await
                    .context("failed to persist vault balances")?;
            }
        }
        Ok(())
    }

    // Transaction operations
    pub async fn insert_transaction(&self, tx: TransactionRecord) -> Result<()> {
        if let Some(pg) = &self.postgres {
            pg.insert_transaction(&tx)
                .await
                .context("failed to persist transaction")?;
        }

        let mut transactions = self.transactions.write().await;
        transactions.push(tx);
        Ok(())
    }

    pub async fn get_user_transactions(&self, user: &str) -> Result<Vec<TransactionRecord>> {
        if let Some(pg) = &self.postgres {
            return pg
                .get_user_transactions(user)
                .await
                .context("failed to load user transactions from postgres");
        }

        let transactions = self.transactions.read().await;
        Ok(transactions
            .iter()
            .filter(|tx| tx.user == user)
            .cloned()
            .collect())
    }

    pub async fn get_all_transactions(&self) -> Result<Vec<TransactionRecord>> {
        if let Some(pg) = &self.postgres {
            return pg
                .get_all_transactions()
                .await
                .context("failed to load transactions from postgres");
        }

        let transactions = self.transactions.read().await;
        Ok(transactions.clone())
    }

    // Balance snapshot operations
    pub async fn insert_snapshot(&self, snapshot: BalanceSnapshot) -> Result<()> {
        if let Some(pg) = &self.postgres {
            pg.create_snapshot(&snapshot)
                .await
                .context("failed to persist balance snapshot")?;
        }

        let mut snapshots = self.balance_snapshots.write().await;
        snapshots.push(snapshot);
        Ok(())
    }

    pub async fn get_snapshots(&self, user: &str) -> Result<Vec<BalanceSnapshot>> {
        if let Some(pg) = &self.postgres {
            return pg
                .get_snapshots(user, 100)
                .await
                .context("failed to load snapshots from postgres");
        }

        let snapshots = self.balance_snapshots.read().await;
        Ok(snapshots
            .iter()
            .filter(|s| s.user == user)
            .cloned()
            .collect())
    }

    // Audit log operations
    pub async fn insert_audit_log(&self, log: AuditLog) -> Result<()> {
        if let Some(pg) = &self.postgres {
            pg.insert_audit_log(&log)
                .await
                .context("failed to persist audit log")?;
        }

        let mut logs = self.audit_logs.write().await;
        logs.push(log);
        Ok(())
    }

    pub async fn get_audit_logs(&self, user: Option<&str>) -> Result<Vec<AuditLog>> {
        if let Some(pg) = &self.postgres {
            return pg
                .get_audit_logs(user, 100)
                .await
                .context("failed to load audit logs from postgres");
        }

        let logs = self.audit_logs.read().await;
        Ok(match user {
            Some(u) => logs.iter().filter(|l| l.user == u).cloned().collect(),
            None => logs.clone(),
        })
    }

    pub async fn log_reconciliation(
        &self,
        user: &str,
        onchain: u64,
        offchain: u64,
        status: &str,
    ) -> Result<()> {
        if let Some(pg) = &self.postgres {
            pg.log_reconciliation(user, onchain as i64, offchain as i64, status)
                .await
                .context("failed to persist reconciliation log")?;
        }
        Ok(())
    }

    pub async fn create_alert(
        &self,
        alert_type: &str,
        severity: &str,
        user: Option<&str>,
        message: &str,
    ) -> Result<()> {
        if let Some(pg) = &self.postgres {
            pg.create_alert(alert_type, severity, user, message)
                .await
                .context("failed to persist alert")?;
        }
        Ok(())
    }

    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        if let Some(pg) = &self.postgres {
            return pg
                .get_active_alerts()
                .await
                .context("failed to load active alerts");
        }

        // Build alerts from in-memory audit log when running without Postgres
        let logs = self.audit_logs.read().await;
        Ok(logs
            .iter()
            .rev()
            .take(10)
            .map(|log| Alert {
                id: Uuid::new_v4().to_string(),
                alert_type: "AUDIT".to_string(),
                severity: "LOW".to_string(),
                user: Some(log.user.clone()),
                message: log.action.clone(),
                timestamp: log.timestamp,
            })
            .collect())
    }
}
