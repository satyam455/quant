use crate::balance_tracker::BalanceTracker;
use crate::db::Database;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetrics {
    pub total_vaults: usize,
    pub total_tvl: u64,
    pub average_balance: u64,
    pub total_transactions: usize,
    pub active_vaults: usize,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub severity: Severity,
    pub message: String,
    pub vault: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct VaultMonitor {
    tracker: Arc<BalanceTracker>,
    db: Arc<Database>,
    metrics_history: Arc<tokio::sync::RwLock<Vec<VaultMetrics>>>,
    security_alerts: Arc<tokio::sync::RwLock<Vec<SecurityAlert>>>,
}

impl VaultMonitor {
    pub fn new(tracker: Arc<BalanceTracker>, db: Arc<Database>) -> Self {
        Self {
            tracker,
            db,
            metrics_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            security_alerts: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Start monitoring service
    pub fn start(self: Arc<Self>) {
        // Metrics collection every 60 seconds
        let monitor_clone = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = monitor_clone.collect_metrics().await {
                    eprintln!("❌ Error collecting metrics: {}", e);
                }
            }
        });

        // Security monitoring every 30 seconds
        let monitor_clone = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = monitor_clone.security_check().await {
                    eprintln!("❌ Error in security check: {}", e);
                }
            }
        });

        println!("🔍 Vault Monitor started");
    }

    /// Collect vault metrics
    async fn collect_metrics(&self) -> Result<()> {
        let tvl = self.tracker.calculate_tvl().await;
        let transactions = self.db.get_all_transactions().await?;

        let metrics = VaultMetrics {
            total_vaults: 0, // You can track this
            total_tvl: tvl,
            average_balance: 0,
            total_transactions: transactions.len(),
            active_vaults: 0,
            timestamp: chrono::Utc::now().timestamp(),
        };

        let mut history = self.metrics_history.write().await;
        history.push(metrics.clone());

        // Keep only last 24 hours of data (1440 minutes)
        let len = history.len();
        if len > 1440 {
            history.drain(0..(len - 1440));
        }

        println!(
            "📊 Metrics collected - TVL: {} tokens, Transactions: {}",
            metrics.total_tvl, metrics.total_transactions
        );

        Ok(())
    }

    /// Security monitoring
    async fn security_check(&self) -> Result<()> {
        let alerts = self.tracker.get_alerts().await;

        for alert in alerts {
            if let crate::balance_tracker::AlertType::UnauthorizedAccess = alert.alert_type {
                let security_alert = SecurityAlert {
                    severity: Severity::Critical,
                    message: alert.message.clone(),
                    vault: Some(alert.vault),
                    timestamp: alert.timestamp,
                };

                let mut security_alerts = self.security_alerts.write().await;
                security_alerts.push(security_alert);

                println!("🚨 SECURITY ALERT: {}", alert.message);
            }
        }

        Ok(())
    }

    /// Get current metrics
    pub async fn get_current_metrics(&self) -> Option<VaultMetrics> {
        let history = self.metrics_history.read().await;
        history.last().cloned()
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self, limit: usize) -> Vec<VaultMetrics> {
        let history = self.metrics_history.read().await;
        let start = if history.len() > limit {
            history.len() - limit
        } else {
            0
        };
        history[start..].to_vec()
    }

    /// Get security alerts
    pub async fn get_security_alerts(&self) -> Vec<SecurityAlert> {
        let alerts = self.security_alerts.read().await;
        alerts.clone()
    }
}
