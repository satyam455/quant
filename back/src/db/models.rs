use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub id: String,
    pub user: String,
    pub tx_type: TransactionType,
    pub amount: u64,
    pub signature: String,
    pub status: TransactionStatus,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Initialize,
    Deposit,
    Withdraw,
    Lock,
    Unlock,
    Transfer,
    WithdrawalRequest,
    WithdrawalExecute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    pub id: String,
    pub user: String,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub user: String,
    pub action: String,
    pub details: String,
    pub ip_address: Option<String>,
    pub timestamp: i64,
}
