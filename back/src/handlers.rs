use crate::vault_manager::VaultManager;
use anchor_lang::prelude::Pubkey;
use axum::{
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

// Helper function to parse pubkey with better error messages
fn parse_pubkey(s: &str) -> Result<Pubkey, String> {
    s.parse::<Pubkey>()
        .map_err(|e| format!("Invalid pubkey '{}': {}", s, e))
}

// --- Endpoints ---
pub async fn register_vault(
    Extension(vm): Extension<Arc<VaultManager>>,
    Json(req): Json<VaultRequest>,
) -> Response {
    let user = match parse_pubkey(&req.user_pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    match vm.initialize_vault(user).await {
        Ok(sig) => (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response(),
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
        Ok(sig) => (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response(),
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
        Ok(sig) => (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response(),
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
        Ok(sig) => (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response(),
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
        Ok(sig) => (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response(),
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
        Ok(sig) => (StatusCode::OK, Json(TxResponse { tx_signature: sig })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}
