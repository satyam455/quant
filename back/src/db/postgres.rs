// src/db/postgres.rs
use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::models::*;

#[derive(Clone)]
pub struct PostgresDatabase {
    pub pool: PgPool,
}

impl PostgresDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn init_schema(&self) -> Result<()> {
        // Run migrations or create tables if they don't exist
        println!("🔧 Running database migrations...");
        let sql_content = include_str!("../../migrations/001_initial_schema.sql");

        // Remove comments and split into statements more robustly
        let cleaned_sql: String = sql_content
            .lines()
            .filter(|line| !line.trim().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");

        // Split by semicolons and execute each statement
        let statements: Vec<&str> = cleaned_sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        println!("📝 Found {} SQL statements to execute", statements.len());

        for (idx, statement) in statements.iter().enumerate() {
            let preview = if statement.len() > 50 {
                format!("{}...", &statement[..50])
            } else {
                statement.to_string()
            };
            println!(
                "   [{}/{}] Executing: {}",
                idx + 1,
                statements.len(),
                preview
            );

            sqlx::query(statement)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to execute SQL statement: {}\nError: {}",
                        &statement[..std::cmp::min(100, statement.len())],
                        e
                    )
                })?;
        }

        println!("✅ Database schema initialized successfully");
        Ok(())
    }

    // ============================================
    // VAULT OPERATIONS
    // ============================================

    pub async fn create_vault(
        &self,
        owner: &str,
        vault_pda: &str,
        token_account: &str,
    ) -> Result<Uuid> {
        let row = sqlx::query(
            r#"
            INSERT INTO vault_accounts 
            (owner_pubkey, vault_pda, token_account, status)
            VALUES ($1, $2, $3, 'ACTIVE')
            RETURNING id
            "#,
        )
        .bind(owner)
        .bind(vault_pda)
        .bind(token_account)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    pub async fn get_vault_by_owner(&self, owner: &str) -> Result<Option<VaultAccount>> {
        let result = sqlx::query_as::<_, VaultAccount>(
            r#"
            SELECT * FROM vault_accounts
            WHERE owner_pubkey = $1
            "#,
        )
        .bind(owner)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn update_vault_balance(
        &self,
        vault_id: Uuid,
        total: i64,
        locked: i64,
        available: i64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE vault_accounts
            SET total_balance = $1,
                locked_balance = $2,
                available_balance = $3,
                updated_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(total)
        .bind(locked)
        .bind(available)
        .bind(vault_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ============================================
    // TRANSACTION OPERATIONS
    // ============================================

    pub async fn insert_transaction(&self, tx: &TransactionRecord) -> Result<Uuid> {
        // First get vault_id from user_pubkey
        let vault = self.get_vault_by_owner(&tx.user).await?;
        let vault_id = vault
            .map(|v| v.id)
            .ok_or_else(|| anyhow::anyhow!("Vault not found"))?;

        let row = sqlx::query(
            r#"
            INSERT INTO transactions 
            (vault_id, user_pubkey, tx_type, amount, signature, status)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(vault_id)
        .bind(&tx.user)
        .bind(format!("{:?}", tx.tx_type))
        .bind(tx.amount as i64)
        .bind(&tx.signature)
        .bind(format!("{:?}", tx.status))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    pub async fn get_user_transactions(&self, user: &str) -> Result<Vec<TransactionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_pubkey, tx_type, amount, signature, status, created_at
            FROM transactions
            WHERE user_pubkey = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user)
        .fetch_all(&self.pool)
        .await?;

        let mut transactions = Vec::new();
        for row in rows {
            let tx_type_str: String = row.get("tx_type");
            let status_str: String = row.get("status");
            let created_at: chrono::NaiveDateTime = row.get("created_at");

            transactions.push(TransactionRecord {
                id: row.get::<Uuid, _>("id").to_string(),
                user: row.get("user_pubkey"),
                tx_type: match tx_type_str.as_str() {
                    "Initialize" => TransactionType::Initialize,
                    "Deposit" => TransactionType::Deposit,
                    "Withdraw" => TransactionType::Withdraw,
                    "Lock" => TransactionType::Lock,
                    "Unlock" => TransactionType::Unlock,
                    "Transfer" => TransactionType::Transfer,
                    "WithdrawalRequest" => TransactionType::WithdrawalRequest,
                    "WithdrawalExecute" => TransactionType::WithdrawalExecute,
                    _ => TransactionType::Deposit,
                },
                amount: row.get::<i64, _>("amount") as u64,
                signature: row.get("signature"),
                status: match status_str.as_str() {
                    "Pending" => TransactionStatus::Pending,
                    "Confirmed" => TransactionStatus::Confirmed,
                    "Failed" => TransactionStatus::Failed,
                    _ => TransactionStatus::Pending,
                },
                timestamp: created_at.and_utc().timestamp(),
            });
        }

        Ok(transactions)
    }

    pub async fn get_all_transactions(&self) -> Result<Vec<TransactionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_pubkey, tx_type, amount, signature, status, created_at
            FROM transactions
            ORDER BY created_at DESC
            LIMIT 1000
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut transactions = Vec::new();
        for row in rows {
            let tx_type_str: String = row.get("tx_type");
            let status_str: String = row.get("status");
            let created_at: chrono::NaiveDateTime = row.get("created_at");

            transactions.push(TransactionRecord {
                id: row.get::<Uuid, _>("id").to_string(),
                user: row.get("user_pubkey"),
                tx_type: match tx_type_str.as_str() {
                    "Initialize" => TransactionType::Initialize,
                    "Deposit" => TransactionType::Deposit,
                    "Withdraw" => TransactionType::Withdraw,
                    "Lock" => TransactionType::Lock,
                    "Unlock" => TransactionType::Unlock,
                    "Transfer" => TransactionType::Transfer,
                    "WithdrawalRequest" => TransactionType::WithdrawalRequest,
                    "WithdrawalExecute" => TransactionType::WithdrawalExecute,
                    _ => TransactionType::Deposit,
                },
                amount: row.get::<i64, _>("amount") as u64,
                signature: row.get("signature"),
                status: match status_str.as_str() {
                    "Pending" => TransactionStatus::Pending,
                    "Confirmed" => TransactionStatus::Confirmed,
                    "Failed" => TransactionStatus::Failed,
                    _ => TransactionStatus::Pending,
                },
                timestamp: created_at.and_utc().timestamp(),
            });
        }

        Ok(transactions)
    }

    // ============================================
    // BALANCE SNAPSHOTS
    // ============================================

    pub async fn create_snapshot(&self, snapshot: &BalanceSnapshot) -> Result<()> {
        let vault = self.get_vault_by_owner(&snapshot.user).await?;
        let vault_id = vault
            .map(|v| v.id)
            .ok_or_else(|| anyhow::anyhow!("Vault not found"))?;

        sqlx::query(
            r#"
            INSERT INTO balance_snapshots 
            (vault_id, user_pubkey, total_balance, locked_balance, available_balance, snapshot_type, snapshot_time)
            VALUES ($1, $2, $3, $4, $5, 'HOURLY', NOW())
            "#
        )
        .bind(vault_id)
        .bind(&snapshot.user)
        .bind(snapshot.total_balance as i64)
        .bind(snapshot.locked_balance as i64)
        .bind(snapshot.available_balance as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_snapshots(&self, user: &str, limit: i64) -> Result<Vec<BalanceSnapshot>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_pubkey, total_balance, locked_balance, available_balance, snapshot_time
            FROM balance_snapshots
            WHERE user_pubkey = $1
            ORDER BY snapshot_time DESC
            LIMIT $2
            "#,
        )
        .bind(user)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut snapshots = Vec::new();
        for row in rows {
            let snapshot_time: chrono::NaiveDateTime = row.get("snapshot_time");
            snapshots.push(BalanceSnapshot {
                id: row.get::<Uuid, _>("id").to_string(),
                user: row.get("user_pubkey"),
                total_balance: row.get::<i64, _>("total_balance") as u64,
                locked_balance: row.get::<i64, _>("locked_balance") as u64,
                available_balance: row.get::<i64, _>("available_balance") as u64,
                timestamp: snapshot_time.and_utc().timestamp(),
            });
        }

        Ok(snapshots)
    }

    // ============================================
    // AUDIT LOGS
    // ============================================

    pub async fn insert_audit_log(&self, log: &AuditLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs 
            (user_pubkey, action, resource_type, details, ip_address)
            VALUES ($1, $2, 'VAULT', $3::jsonb, $4)
            "#,
        )
        .bind(&log.user)
        .bind(&log.action)
        .bind(&log.details)
        .bind(log.ip_address.as_deref())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_audit_logs(&self, user: Option<&str>, limit: i64) -> Result<Vec<AuditLog>> {
        let rows = if let Some(u) = user {
            sqlx::query(
                r#"
                SELECT id, user_pubkey, action, details, ip_address, created_at
                FROM audit_logs
                WHERE user_pubkey = $1
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(u)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, user_pubkey, action, details, ip_address, created_at
                FROM audit_logs
                ORDER BY created_at DESC
                LIMIT $1
                "#,
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        let mut logs = Vec::new();
        for row in rows {
            let created_at: chrono::NaiveDateTime = row.get("created_at");
            logs.push(AuditLog {
                id: row.get::<Uuid, _>("id").to_string(),
                user: row.get("user_pubkey"),
                action: row.get("action"),
                details: row.get("details"),
                ip_address: row.get("ip_address"),
                timestamp: created_at.and_utc().timestamp(),
            });
        }

        Ok(logs)
    }

    // ============================================
    // ANALYTICS
    // ============================================

    pub async fn get_tvl(&self) -> Result<i64> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(total_balance), 0) as tvl
            FROM vault_accounts
            WHERE status = 'ACTIVE'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("tvl"))
    }

    pub async fn get_active_vaults_count(&self) -> Result<i64> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM vault_accounts
            WHERE status = 'ACTIVE'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("count"))
    }

    // ============================================
    // RECONCILIATION
    // ============================================

    pub async fn log_reconciliation(
        &self,
        user: &str,
        onchain: i64,
        offchain: i64,
        status: &str,
    ) -> Result<()> {
        let vault = self.get_vault_by_owner(user).await?;
        let vault_id = vault
            .map(|v| v.id)
            .ok_or_else(|| anyhow::anyhow!("Vault not found"))?;

        let discrepancy = onchain - offchain;

        sqlx::query(
            r#"
            INSERT INTO reconciliation_logs 
            (vault_id, user_pubkey, onchain_balance, offchain_balance, discrepancy, status)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(vault_id)
        .bind(user)
        .bind(onchain)
        .bind(offchain)
        .bind(discrepancy)
        .bind(status)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ============================================
    // ALERTS
    // ============================================

    pub async fn create_alert(
        &self,
        alert_type: &str,
        severity: &str,
        user: Option<&str>,
        message: &str,
    ) -> Result<()> {
        let vault_id = if let Some(u) = user {
            self.get_vault_by_owner(u).await?.map(|v| v.id)
        } else {
            None
        };

        sqlx::query(
            r#"
            INSERT INTO alerts 
            (alert_type, severity, vault_id, user_pubkey, message, status)
            VALUES ($1, $2, $3, $4, $5, 'ACTIVE')
            "#,
        )
        .bind(alert_type)
        .bind(severity)
        .bind(vault_id)
        .bind(user)
        .bind(message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        let rows = sqlx::query(
            r#"
            SELECT id, alert_type, severity, user_pubkey, message, created_at
            FROM alerts
            WHERE status = 'ACTIVE'
            ORDER BY created_at DESC
            LIMIT 100
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut alerts = Vec::new();
        for row in rows {
            let created_at: chrono::NaiveDateTime = row.get("created_at");
            alerts.push(Alert {
                id: row.get::<Uuid, _>("id").to_string(),
                alert_type: row.get("alert_type"),
                severity: row.get("severity"),
                user: row.get("user_pubkey"),
                message: row.get("message"),
                timestamp: created_at.and_utc().timestamp(),
            });
        }

        Ok(alerts)
    }
}

// ============================================
// MODELS FOR POSTGRES
// ============================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VaultAccount {
    pub id: Uuid,
    pub owner_pubkey: String,
    pub vault_pda: String,
    pub token_account: String,
    pub total_balance: i64,
    pub locked_balance: i64,
    pub available_balance: i64,
    pub total_deposited: i64,
    pub total_withdrawn: i64,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub alert_type: String,
    pub severity: String,
    pub user: Option<String>,
    pub message: String,
    pub timestamp: i64,
}
