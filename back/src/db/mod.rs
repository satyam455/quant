use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod models;
pub mod postgres;

pub use models::*;
pub use postgres::PostgresDatabase;

// Simple in-memory database (replace with PostgreSQL later)
pub struct Database {
    transactions: Arc<RwLock<Vec<TransactionRecord>>>,
    balance_snapshots: Arc<RwLock<Vec<BalanceSnapshot>>>,
    audit_logs: Arc<RwLock<Vec<AuditLog>>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(Vec::new())),
            balance_snapshots: Arc::new(RwLock::new(Vec::new())),
            audit_logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // Transaction operations
    pub async fn insert_transaction(&self, tx: TransactionRecord) -> Result<()> {
        let mut transactions = self.transactions.write().await;
        transactions.push(tx);
        Ok(())
    }

    pub async fn get_user_transactions(&self, user: &str) -> Result<Vec<TransactionRecord>> {
        let transactions = self.transactions.read().await;
        Ok(transactions
            .iter()
            .filter(|tx| tx.user == user)
            .cloned()
            .collect())
    }

    pub async fn get_all_transactions(&self) -> Result<Vec<TransactionRecord>> {
        let transactions = self.transactions.read().await;
        Ok(transactions.clone())
    }

    // Balance snapshot operations
    pub async fn insert_snapshot(&self, snapshot: BalanceSnapshot) -> Result<()> {
        let mut snapshots = self.balance_snapshots.write().await;
        snapshots.push(snapshot);
        Ok(())
    }

    pub async fn get_snapshots(&self, user: &str) -> Result<Vec<BalanceSnapshot>> {
        let snapshots = self.balance_snapshots.read().await;
        Ok(snapshots
            .iter()
            .filter(|s| s.user == user)
            .cloned()
            .collect())
    }

    // Audit log operations
    pub async fn insert_audit_log(&self, log: AuditLog) -> Result<()> {
        let mut logs = self.audit_logs.write().await;
        logs.push(log);
        Ok(())
    }

    pub async fn get_audit_logs(&self, user: Option<&str>) -> Result<Vec<AuditLog>> {
        let logs = self.audit_logs.read().await;
        Ok(match user {
            Some(u) => logs.iter().filter(|l| l.user == u).cloned().collect(),
            None => logs.clone(),
        })
    }
}
