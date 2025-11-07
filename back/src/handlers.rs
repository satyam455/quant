use crate::analytics::AnalyticsService;
use crate::balance_tracker::{BalanceAlert, BalanceTracker, VaultBalance};
use crate::db::{AuditLog, Database, TransactionRecord, TransactionStatus, TransactionType};
use crate::vault_manager::VaultManager;
use crate::websocket::{WebSocketManager, WsMessage};
use anchor_lang::prelude::Pubkey;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct VaultRequest {
    pub user_pubkey: String,
    pub amount: Option<u64>,
    pub to_pubkey: Option<String>,
    pub request_id: Option<u64>,
    pub authorized_programs: Option<Vec<String>>,
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
    pub transactions: Vec<TransactionRecord>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct AlertsResponse {
    pub alerts: Vec<BalanceAlert>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

fn parse_pubkey(s: &str) -> Result<Pubkey, String> {
    s.parse::<Pubkey>()
        .map_err(|e| format!("Invalid pubkey '{}': {}", s, e))
}

async fn record_transaction(
    db: &Arc<Database>,
    user: &str,
    tx_type: TransactionType,
    amount: u64,
    signature: &str,
    action: &str,
    details: &str,
) {
    let timestamp = Utc::now().timestamp();
    let record = TransactionRecord {
        id: Uuid::new_v4().to_string(),
        user: user.to_string(),
        tx_type,
        amount,
        signature: signature.to_string(),
        status: TransactionStatus::Confirmed,
        timestamp,
    };
    if let Err(err) = db.insert_transaction(record).await {
        eprintln!("⚠️  Failed to store transaction: {}", err);
    }

    let audit = AuditLog {
        id: Uuid::new_v4().to_string(),
        user: user.to_string(),
        action: action.to_string(),
        details: details.to_string(),
        ip_address: None,
        timestamp,
    };
    if let Err(err) = db.insert_audit_log(audit).await {
        eprintln!("⚠️  Failed to store audit log: {}", err);
    }
}

async fn refresh_balance(tracker: &Arc<BalanceTracker>, ws: &Arc<WebSocketManager>, user: Pubkey) {
    if let Ok(balance) = tracker.get_vault_balance(user).await {
        ws.broadcast(WsMessage::BalanceUpdate {
            user: balance.owner.clone(),
            balance: balance.available_balance,
        });
    }
}

pub async fn health_check() -> Response {
    (StatusCode::OK, Json(HealthResponse { status: "ok" })).into_response()
}

pub async fn initialize_vault(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Extension(tracker): Extension<Arc<BalanceTracker>>,
    Extension(ws): Extension<Arc<WebSocketManager>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    let authorized_programs = req
        .authorized_programs
        .unwrap_or_else(|| vec![vm.program.id().to_string()])
        .into_iter()
        .filter_map(|p| parse_pubkey(&p).ok())
        .collect::<Vec<_>>();

    match vm.initialize_vault(user, authorized_programs.clone()).await {
        Ok(sig) => {
            let (vault_pda, _) =
                Pubkey::find_program_address(&[b"vault", user.as_ref()], &vm.program.id());
            let (vault_token, _) =
                Pubkey::find_program_address(&[b"vault_token", user.as_ref()], &vm.program.id());
            let _ = db.register_vault(&user, &vault_pda, &vault_token).await;

            record_transaction(
                &db,
                &req.user_pubkey,
                TransactionType::Initialize,
                0,
                &sig,
                "INITIALIZE_VAULT",
                &format!("Vault initialized: {}", sig),
            )
            .await;

            refresh_balance(&tracker, &ws, user).await;

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

pub async fn deposit_collateral(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Extension(tracker): Extension<Arc<BalanceTracker>>,
    Extension(ws): Extension<Arc<WebSocketManager>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };
    let amount = match req.amount {
        Some(amt) if amt > 0 => amt,
        _ => {
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
            record_transaction(
                &db,
                &req.user_pubkey,
                TransactionType::Deposit,
                amount,
                &sig,
                "DEPOSIT",
                &format!("Deposit successful: {}", sig),
            )
            .await;

            refresh_balance(&tracker, &ws, user).await;
            ws.broadcast(WsMessage::DepositNotification {
                user: req.user_pubkey.clone(),
                amount,
                signature: sig.clone(),
            });

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

pub async fn withdraw_collateral(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Extension(tracker): Extension<Arc<BalanceTracker>>,
    Extension(ws): Extension<Arc<WebSocketManager>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };
    let amount = match req.amount {
        Some(amt) if amt > 0 => amt,
        _ => {
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
            record_transaction(
                &db,
                &req.user_pubkey,
                TransactionType::Withdraw,
                amount,
                &sig,
                "WITHDRAW",
                &format!("Withdrawal successful: {}", sig),
            )
            .await;

            refresh_balance(&tracker, &ws, user).await;
            ws.broadcast(WsMessage::WithdrawNotification {
                user: req.user_pubkey.clone(),
                amount,
                signature: sig.clone(),
            });

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

pub async fn request_withdrawal(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Extension(tracker): Extension<Arc<BalanceTracker>>,
    Extension(ws): Extension<Arc<WebSocketManager>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };
    let amount = match req.amount {
        Some(amt) if amt > 0 => amt,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "amount is required".to_string(),
                }),
            )
                .into_response()
        }
    };
    let request_id = match req.request_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "request_id is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    match vm.request_withdrawal(user, request_id, amount).await {
        Ok(sig) => {
            record_transaction(
                &db,
                &req.user_pubkey,
                TransactionType::WithdrawalRequest,
                amount,
                &sig,
                "WITHDRAWAL_REQUEST",
                &format!("Withdrawal request created: {}", sig),
            )
            .await;

            refresh_balance(&tracker, &ws, user).await;

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

pub async fn execute_withdrawal(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Extension(tracker): Extension<Arc<BalanceTracker>>,
    Extension(ws): Extension<Arc<WebSocketManager>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };
    let request_id = match req.request_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "request_id is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    match vm.execute_withdrawal(user, request_id).await {
        Ok(sig) => {
            record_transaction(
                &db,
                &req.user_pubkey,
                TransactionType::WithdrawalExecute,
                0,
                &sig,
                "WITHDRAWAL_EXECUTE",
                &format!("Withdrawal executed: {}", sig),
            )
            .await;

            refresh_balance(&tracker, &ws, user).await;

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

pub async fn lock_collateral(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Extension(tracker): Extension<Arc<BalanceTracker>>,
    Extension(ws): Extension<Arc<WebSocketManager>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };
    let amount = match req.amount {
        Some(amt) if amt > 0 => amt,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "amount is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    let authority_program = vm.program.id();

    match vm.lock(user, authority_program, amount).await {
        Ok(sig) => {
            record_transaction(
                &db,
                &req.user_pubkey,
                TransactionType::Lock,
                amount,
                &sig,
                "LOCK",
                &format!("Collateral locked: {}", sig),
            )
            .await;

            refresh_balance(&tracker, &ws, user).await;

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

pub async fn unlock_collateral(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Extension(tracker): Extension<Arc<BalanceTracker>>,
    Extension(ws): Extension<Arc<WebSocketManager>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };
    let amount = match req.amount {
        Some(amt) if amt > 0 => amt,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "amount is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    let authority_program = vm.program.id();

    match vm.unlock(user, authority_program, amount).await {
        Ok(sig) => {
            record_transaction(
                &db,
                &req.user_pubkey,
                TransactionType::Unlock,
                amount,
                &sig,
                "UNLOCK",
                &format!("Collateral unlocked: {}", sig),
            )
            .await;

            refresh_balance(&tracker, &ws, user).await;

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

pub async fn transfer_collateral(
    Extension(vm): Extension<Arc<VaultManager>>,
    Extension(db): Extension<Arc<Database>>,
    Extension(tracker): Extension<Arc<BalanceTracker>>,
    Extension(ws): Extension<Arc<WebSocketManager>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let from = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };
    let to_pubkey = match &req.to_pubkey {
        Some(pk) => match parse_pubkey(pk) {
            Ok(v) => v,
            Err(e) => {
                return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
            }
        },
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
    let amount = match req.amount {
        Some(amt) if amt > 0 => amt,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "amount is required".to_string(),
                }),
            )
                .into_response()
        }
    };

    let authority_program = vm.program.id();

    match vm
        .transfer(from, to_pubkey, authority_program, amount)
        .await
    {
        Ok(sig) => {
            record_transaction(
                &db,
                &req.user_pubkey,
                TransactionType::Transfer,
                amount,
                &sig,
                "TRANSFER",
                &format!("Internal transfer: {}", sig),
            )
            .await;

            refresh_balance(&tracker, &ws, from).await;
            refresh_balance(&tracker, &ws, to_pubkey).await;

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

pub async fn get_tvl(Extension(tracker): Extension<Arc<BalanceTracker>>) -> Response {
    let tvl = tracker.calculate_tvl().await;
    (
        StatusCode::OK,
        Json(TvlResponse {
            total_value_locked: tvl,
            total_vaults: 0,
            timestamp: Utc::now().timestamp(),
        }),
    )
        .into_response()
}

pub async fn get_alerts(Extension(tracker): Extension<Arc<BalanceTracker>>) -> Response {
    let alerts = tracker.get_alerts().await;
    let count = alerts.len();
    (StatusCode::OK, Json(AlertsResponse { alerts, count })).into_response()
}

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
                balance: VaultBalance,
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
