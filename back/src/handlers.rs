use crate::analytics::AnalyticsService;
use crate::balance_tracker::BalanceTracker;
use crate::db::Database;
use crate::vault_manager::VaultManager;
use anchor_lang::prelude::Pubkey;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct VaultRequest {
    pub user_pubkey: String,
    pub amount: Option<u64>,
    pub to_pubkey: Option<String>,
}

#[derive(Serialize)]
pub struct TxResponse {
    pub tx_signature: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    pub owner: String,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct TvlResponse {
    pub total_value_locked: u64,
    pub total_vaults: usize,
    pub timestamp: i64,
}

#[derive(Serialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<crate::db::TransactionRecord>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct AlertsResponse {
    pub alerts: Vec<crate::balance_tracker::BalanceAlert>,
    pub count: usize,
}

// Helper function to parse pubkey
fn parse_pubkey(s: &str) -> Result<Pubkey, String> {
    s.parse::<Pubkey>()
        .map_err(|e| format!("Invalid pubkey '{}': {}", s, e))
}

// ===== POST Endpoints (Existing) =====

pub async fn register_vault(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    match vm.initialize_vault(user).await {
        Ok(sig) => {
            // Log transaction
            let _ = db
                .insert_transaction(crate::db::TransactionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    user: req.user_pubkey.clone(),
                    tx_type: crate::db::TransactionType::Initialize,
                    amount: 0,
                    signature: sig.clone(),
                    status: crate::db::TransactionStatus::Confirmed,
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .await;

            // Audit log
            let _ = db
                .insert_audit_log(crate::db::AuditLog {
                    id: uuid::Uuid::new_v4().to_string(),
                    user: req.user_pubkey,
                    action: "INITIALIZE_VAULT".to_string(),
                    details: format!("Vault initialized with signature: {}", sig),
                    ip_address: None,
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .await;

            (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn deposit(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    let amount = match req.amount {
        Some(amt) => amt,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "amount is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    match vm.deposit(user, amount).await {
        Ok(sig) => {
            // Log transaction
            let _ = db
                .insert_transaction(crate::db::TransactionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    user: req.user_pubkey.clone(),
                    tx_type: crate::db::TransactionType::Deposit,
                    amount,
                    signature: sig.clone(),
                    status: crate::db::TransactionStatus::Confirmed,
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .await;

            (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn withdraw(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    let amount = match req.amount {
        Some(amt) => amt,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "amount is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    match vm.withdraw(user, amount).await {
        Ok(sig) => {
            // Log transaction
            let _ = db
                .insert_transaction(crate::db::TransactionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    user: req.user_pubkey.clone(),
                    tx_type: crate::db::TransactionType::Withdraw,
                    amount,
                    signature: sig.clone(),
                    status: crate::db::TransactionStatus::Confirmed,
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .await;

            (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn lock(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    let amount = match req.amount {
        Some(amt) => amt,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "amount is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    match vm.lock(user, amount).await {
        Ok(sig) => {
            // Log transaction
            let _ = db
                .insert_transaction(crate::db::TransactionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    user: req.user_pubkey.clone(),
                    tx_type: crate::db::TransactionType::Lock,
                    amount,
                    signature: sig.clone(),
                    status: crate::db::TransactionStatus::Confirmed,
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .await;

            (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn unlock(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    let amount = match req.amount {
        Some(amt) => amt,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "amount is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    match vm.unlock(user, amount).await {
        Ok(sig) => {
            // Log transaction
            let _ = db
                .insert_transaction(crate::db::TransactionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    user: req.user_pubkey.clone(),
                    tx_type: crate::db::TransactionType::Unlock,
                    amount,
                    signature: sig.clone(),
                    status: crate::db::TransactionStatus::Confirmed,
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .await;

            (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn transfer(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let from = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    let to_str = match &req.to_pubkey {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "to_pubkey is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    let to = match parse_pubkey(to_str) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    let amount = match req.amount {
        Some(amt) => amt,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "amount is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    match vm.transfer(from, to, amount).await {
        Ok(sig) => {
            // Log transaction
            let _ = db
                .insert_transaction(crate::db::TransactionRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    user: req.user_pubkey.clone(),
                    tx_type: crate::db::TransactionType::Transfer,
                    amount,
                    signature: sig.clone(),
                    status: crate::db::TransactionStatus::Confirmed,
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .await;

            (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

// ===== NEW GET Endpoints =====

/// GET /vault/balance/:user - Get vault balance
pub async fn get_balance(
    Path(user_pubkey): Path<String>,
    Extension(tracker): Extension<Arc<BalanceTracker>>,
) -> Response {
    let user = match parse_pubkey(&user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    match tracker.get_vault_balance(user).await {
        Ok(balance) => (StatusCode::OK, Json(balance)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// GET /vault/transactions/:user - Get transaction history
pub async fn get_transactions(
    Path(user_pubkey): Path<String>,
    Extension(db): Extension<Arc<Database>>,
) -> Response {
    match db.get_user_transactions(&user_pubkey).await {
        Ok(transactions) => {
            let count = transactions.len();
            (
                StatusCode::OK,
                Json(TransactionsResponse {
                    transactions,
                    count,
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// GET /vault/tvl - Get total value locked
pub async fn get_tvl(Extension(tracker): Extension<Arc<BalanceTracker>>) -> Response {
    let tvl = tracker.calculate_tvl().await;
    (
        StatusCode::OK,
        Json(TvlResponse {
            total_value_locked: tvl,
            total_vaults: 0, // You can track this separately
            timestamp: chrono::Utc::now().timestamp(),
        }),
    )
        .into_response()
}

/// GET /vault/alerts - Get system alerts
pub async fn get_alerts(Extension(tracker): Extension<Arc<BalanceTracker>>) -> Response {
    let alerts = tracker.get_alerts().await;
    let count = alerts.len();
    (StatusCode::OK, Json(AlertsResponse { alerts, count })).into_response()
}

/// GET /vault/status/:user - Get vault status
pub async fn get_vault_status(
    Path(user_pubkey): Path<String>,
    Extension(tracker): Extension<Arc<BalanceTracker>>,
    Extension(db): Extension<Arc<Database>>,
) -> Response {
    let user = match parse_pubkey(&user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    match tracker.get_vault_balance(user).await {
        Ok(balance) => {
            let transactions = db
                .get_user_transactions(&user_pubkey)
                .await
                .unwrap_or_default();

            #[derive(Serialize)]
            struct StatusResponse {
                balance: crate::balance_tracker::VaultBalance,
                transaction_count: usize,
                health_score: f64,
            }

            let health_score = if balance.total_balance > 0 {
                (balance.available_balance as f64 / balance.total_balance as f64) * 100.0
            } else {
                100.0
            };

            (
                StatusCode::OK,
                Json(StatusResponse {
                    balance,
                    transaction_count: transactions.len(),
                    health_score,
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// GET /analytics/dashboard - System analytics
pub async fn get_dashboard_analytics(
    Extension(analytics): Extension<Arc<AnalyticsService>>,
) -> Response {
    match analytics.get_system_analytics().await {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// GET /analytics/tvl-history/:days
pub async fn get_tvl_history(
    Path(days): Path<i64>,
    Extension(analytics): Extension<Arc<AnalyticsService>>,
) -> Response {
    match analytics.get_tvl_history(days).await {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}
