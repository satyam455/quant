use crate::db::postgres::PostgresDatabase;
use anyhow::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row; // Add this import

#[derive(Debug, Serialize, Deserialize)]
pub struct TvlHistory {
    pub timestamp: i64,
    pub tvl: i64,
    pub vault_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMetrics {
    pub user: String,
    pub total_deposits: i64,
    pub total_withdrawals: i64,
    pub current_balance: i64,
    pub position_count: i64,
    pub avg_locked_ratio: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemAnalytics {
    pub tvl_7d: Vec<TvlHistory>,
    pub top_users: Vec<UserMetrics>,
    pub total_volume_24h: i64,
    pub active_vaults: i64,
    pub total_transactions_24h: i64,
}

pub struct AnalyticsService {
    db: PostgresDatabase,
}

impl AnalyticsService {
    pub fn new(db: PostgresDatabase) -> Self {
        Self { db }
    }

    pub async fn get_tvl_history(&self, days: i64) -> Result<Vec<TvlHistory>> {
        let cutoff = Utc::now() - Duration::days(days);

        let rows = sqlx::query(
            r#"
            SELECT 
                DATE_TRUNC('hour', snapshot_time) as hour,
                SUM(total_balance) as tvl,
                COUNT(DISTINCT vault_id) as vault_count
            FROM balance_snapshots
            WHERE snapshot_time >= $1
            GROUP BY hour
            ORDER BY hour DESC
            "#,
        )
        .bind(cutoff.naive_utc())
        .fetch_all(&self.db.pool)
        .await?;

        let mut history = Vec::new();
        for row in rows {
            let hour: chrono::NaiveDateTime = row.try_get("hour")?;
            let tvl: i64 = row.try_get("tvl").unwrap_or(0);
            let vault_count: i64 = row.try_get("vault_count").unwrap_or(0);

            history.push(TvlHistory {
                timestamp: hour.and_utc().timestamp(),
                tvl,
                vault_count,
            });
        }

        Ok(history)
    }

    pub async fn get_top_users(&self, limit: i64) -> Result<Vec<UserMetrics>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                v.owner_pubkey as user,
                v.total_deposited,
                v.total_withdrawn,
                v.total_balance as current_balance,
                CASE WHEN v.total_balance > 0 
                     THEN (v.locked_balance::float / v.total_balance::float) * 100 
                     ELSE 0 
                END as avg_locked_ratio
            FROM vault_accounts v
            WHERE v.status = 'ACTIVE'
            ORDER BY v.total_balance DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.db.pool)
        .await?;

        let mut users = Vec::new();
        for row in rows {
            users.push(UserMetrics {
                user: row.try_get("user").unwrap_or_default(),
                total_deposits: row.try_get("total_deposited").unwrap_or(0),
                total_withdrawals: row.try_get("total_withdrawn").unwrap_or(0),
                current_balance: row.try_get("current_balance").unwrap_or(0),
                position_count: 0,
                avg_locked_ratio: row.try_get("avg_locked_ratio").unwrap_or(0.0),
            });
        }

        Ok(users)
    }

    pub async fn get_system_analytics(&self) -> Result<SystemAnalytics> {
        let tvl_7d = self.get_tvl_history(7).await?;
        let top_users = self.get_top_users(10).await?;

        let volume_row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(amount), 0) as volume
            FROM transactions
            WHERE created_at >= NOW() - INTERVAL '24 hours'
            AND tx_type IN ('Deposit', 'Withdraw')
            "#,
        )
        .fetch_one(&self.db.pool)
        .await?;

        let active_vaults = self.db.get_active_vaults_count().await?;

        let tx_row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM transactions
            WHERE created_at >= NOW() - INTERVAL '24 hours'
            "#,
        )
        .fetch_one(&self.db.pool)
        .await?;

        Ok(SystemAnalytics {
            tvl_7d,
            top_users,
            total_volume_24h: volume_row.try_get("volume").unwrap_or(0),
            active_vaults,
            total_transactions_24h: tx_row.try_get("count").unwrap_or(0),
        })
    }
}
