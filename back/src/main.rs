use anyhow::{anyhow, Context, Result};
use axum::{
    routing::{get, post},
    Extension, Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;

mod analytics;
mod balance_tracker;
mod config;
mod cpi_manager;
mod db;
mod handlers;
mod vault_manager;
mod vault_monitor;
mod websocket;

use analytics::AnalyticsService;
use balance_tracker::BalanceTracker;
use config::AppConfig;
use cpi_manager::CPIManager;
use db::{postgres::PostgresDatabase, Database};
use vault_monitor::VaultMonitor;
use websocket::WebSocketManager;

#[tokio::main]
async fn main() -> Result<()> {
    let AppConfig {
        rpc_url,
        program_id,
        usdt_mint,
        database_url,
        bind_addr,
        payer,
        user,
    } = AppConfig::from_env()?;

    // Create VaultManager in a blocking context to avoid runtime conflicts
    // anchor-client uses synchronous RPC calls that can conflict with tokio runtime
    let vault_mgr = tokio::task::spawn_blocking(move || {
        vault_manager::VaultManager::new(rpc_url, payer, user, program_id, usdt_mint)
    })
    .await
    .map_err(|err| anyhow!("Failed to join VaultManager task: {err}"))??;
    let vault_mgr = Arc::new(vault_mgr);

    println!(
        "✅ VaultManager ready - program: {} mint: {}",
        program_id, usdt_mint
    );

    println!("🔗 Connecting to database: {}", database_url);
    let postgres = match PostgresDatabase::new(&database_url).await {
        Ok(pool) => {
            println!("✅ Database connected successfully");
            pool
        }
        Err(err) => {
            eprintln!("❌ Failed to connect to PostgreSQL: {err}");
            return Err(err.into());
        }
    };

    if let Err(err) = postgres.init_schema().await {
        eprintln!("❌ Failed to run database migrations: {err}");
        return Err(err.into());
    }

    let database = Arc::new(Database::new(Some(postgres.clone())));
    let balance_tracker = Arc::new(BalanceTracker::new(vault_mgr.clone(), database.clone()));
    let vault_monitor = Arc::new(VaultMonitor::new(balance_tracker.clone(), database.clone()));
    let analytics = Arc::new(AnalyticsService::new(postgres));
    let ws_manager = Arc::new(WebSocketManager::new());
    let _cpi_manager = Arc::new(CPIManager::new(vault_mgr.clone()));

    balance_tracker.clone().start_monitoring();
    vault_monitor.clone().start();

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/vault/initialize", post(handlers::initialize_vault))
        .route("/vault/deposit", post(handlers::deposit_collateral))
        .route("/vault/withdraw", post(handlers::withdraw_collateral))
        .route(
            "/vault/request-withdrawal",
            post(handlers::request_withdrawal),
        )
        .route(
            "/vault/execute-withdrawal",
            post(handlers::execute_withdrawal),
        )
        .route("/vault/lock", post(handlers::lock_collateral))
        .route("/vault/unlock", post(handlers::unlock_collateral))
        .route("/vault/transfer", post(handlers::transfer_collateral))
        .route("/vault/balance/{user}", get(handlers::get_balance))
        .route(
            "/vault/transactions/{user}",
            get(handlers::get_transactions),
        )
        .route("/vault/status/{user}", get(handlers::get_vault_status))
        .route("/vault/tvl", get(handlers::get_tvl))
        .route("/vault/alerts", get(handlers::get_alerts))
        .route(
            "/analytics/dashboard",
            get(handlers::get_dashboard_analytics),
        )
        .route(
            "/analytics/tvl-history/{days}",
            get(handlers::get_tvl_history),
        )
        .route("/ws", get(websocket::ws_handler))
        .layer(Extension(vault_mgr))
        .layer(Extension(balance_tracker))
        .layer(Extension(database))
        .layer(Extension(analytics))
        .layer(Extension(ws_manager));

    let listener = TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("Failed to bind to {}", bind_addr))?;

    println!("\n🚀 Vault Backend Server Started at http://{}", bind_addr);
    println!("📡 REST API:");
    println!("   POST /vault/initialize");
    println!("   POST /vault/deposit");
    println!("   POST /vault/withdraw");
    println!("   POST /vault/request-withdrawal");
    println!("   POST /vault/execute-withdrawal");
    println!("   POST /vault/lock");
    println!("   POST /vault/unlock");
    println!("   POST /vault/transfer");
    println!("   GET  /vault/balance/{{user}}");
    println!("   GET  /vault/transactions/{{user}}");
    println!("   GET  /vault/status/{{user}}");
    println!("   GET  /vault/tvl");
    println!("   GET  /vault/alerts");
    println!("   GET  /analytics/dashboard");
    println!("   GET  /analytics/tvl-history/{{days}}");
    println!("   GET  /health");
    println!("📡 WebSocket: /ws");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;
    let _ = signal::ctrl_c().await;
    println!("\n🛑 Shutting down gracefully...");
}
