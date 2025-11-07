use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use axum::{
    routing::{get, post},
    Extension, Router,
};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

mod analytics;
mod balance_tracker;
mod cpi_manager;
mod db;
mod handlers;
mod vault_manager;
mod vault_monitor;
mod websocket;

use analytics::AnalyticsService;
use balance_tracker::BalanceTracker;
use cpi_manager::CPIManager;
use db::{postgres::PostgresDatabase, Database};
use vault_monitor::VaultMonitor;
use websocket::WebSocketManager;

#[tokio::main]
async fn main() -> Result<()> {
    let payer = read_keypair_file("/home/satyam/.config/solana/id.json")
        .expect("Failed to read Solana keypair");

    let program_id: Pubkey = "4NYik8PfZkQdj89AjVxX8LWHZyPKWQ647XKNpCMk6gAR"
        .parse()
        .unwrap();

    let usdt_mint: Pubkey = "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"
        .parse()
        .expect("Invalid USDT mint address");

    let vault_mgr = Arc::new(
        vault_manager::VaultManager::new(
            "https://api.devnet.solana.com".to_string(),
            payer,
            program_id,
            usdt_mint,
        )
        .expect("❌ Failed to create VaultManager"),
    );

    println!("✅ VaultManager initialized successfully");
    println!("📍 Program ID: {}", program_id);
    println!("💵 USDT Mint: {}", usdt_mint);

    // Initialize all components
    let balance_tracker = Arc::new(BalanceTracker::new(vault_mgr.clone()));
    let database = Arc::new(Database::new());
    let cpi_manager = Arc::new(CPIManager::new(vault_mgr.clone()));
    let vault_monitor = Arc::new(VaultMonitor::new(balance_tracker.clone(), database.clone()));
    let ws_manager = Arc::new(WebSocketManager::new());

    // Start background services (now inside tokio runtime)
    balance_tracker.clone().start_monitoring();
    vault_monitor.clone().start();

    // Initialize PostgreSQL database
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://vault_user:secure_password_123@localhost/vault_management".to_string()
    });

    println!("🔗 Connecting to database: {}", database_url);

    let pg_db = match PostgresDatabase::new(&database_url).await {
        Ok(db) => {
            println!("✅ Database connected successfully");
            Some(db)
        }
        Err(e) => {
            eprintln!("⚠️  Database connection failed: {}", e);
            eprintln!("   Running without PostgreSQL analytics");
            None
        }
    };

    let app = if let Some(pg_db) = pg_db {
        let analytics = Arc::new(AnalyticsService::new(pg_db));

        Router::new()
            // POST endpoints
            .route("/register", post(handlers::register_vault))
            .route("/deposit", post(handlers::deposit))
            .route("/withdraw", post(handlers::withdraw))
            .route("/lock", post(handlers::lock))
            .route("/unlock", post(handlers::unlock))
            .route("/transfer", post(handlers::transfer))
            // GET endpoints
            .route("/vault/balance/:user", get(handlers::get_balance))
            .route("/vault/transactions/:user", get(handlers::get_transactions))
            .route("/vault/status/:user", get(handlers::get_vault_status))
            .route("/vault/tvl", get(handlers::get_tvl))
            .route("/vault/alerts", get(handlers::get_alerts))
            // Analytics endpoints
            .route(
                "/analytics/dashboard",
                get(handlers::get_dashboard_analytics),
            )
            .route(
                "/analytics/tvl-history/:days",
                get(handlers::get_tvl_history),
            )
            // WebSocket endpoint
            .route("/ws", get(websocket::ws_handler))
            .layer(Extension(vault_mgr))
            .layer(Extension(balance_tracker))
            .layer(Extension(database))
            .layer(Extension(analytics))
            .layer(Extension(ws_manager))
    } else {
        Router::new()
            // POST endpoints
            .route("/register", post(handlers::register_vault))
            .route("/deposit", post(handlers::deposit))
            .route("/withdraw", post(handlers::withdraw))
            .route("/lock", post(handlers::lock))
            .route("/unlock", post(handlers::unlock))
            .route("/transfer", post(handlers::transfer))
            // GET endpoints
            .route("/vault/balance/:user", get(handlers::get_balance))
            .route("/vault/transactions/:user", get(handlers::get_transactions))
            .route("/vault/status/:user", get(handlers::get_vault_status))
            .route("/vault/tvl", get(handlers::get_tvl))
            .route("/vault/alerts", get(handlers::get_alerts))
            // WebSocket endpoint
            .route("/ws", get(websocket::ws_handler))
            .layer(Extension(vault_mgr))
            .layer(Extension(balance_tracker))
            .layer(Extension(database))
            .layer(Extension(ws_manager))
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;

    println!("\n🚀 =================================");
    println!("   Vault Backend Server Started");
    println!("   =================================");
    println!("\n📍 Server: http://{}", addr);
    println!("\n📡 REST API Endpoints:");
    println!("\n   POST Endpoints:");
    println!("   ├─ /register               - Initialize vault");
    println!("   ├─ /deposit                - Deposit collateral");
    println!("   ├─ /withdraw               - Withdraw collateral");
    println!("   ├─ /lock                   - Lock collateral");
    println!("   ├─ /unlock                 - Unlock collateral");
    println!("   └─ /transfer               - Transfer collateral");
    println!("\n   GET Endpoints:");
    println!("   ├─ /vault/balance/:user    - Get vault balance");
    println!("   ├─ /vault/transactions/:user - Get transaction history");
    println!("   ├─ /vault/status/:user     - Get vault status");
    println!("   ├─ /vault/tvl              - Get total value locked");
    println!("   └─ /vault/alerts           - Get system alerts");
    println!("\n   Analytics:");
    println!("   ├─ /analytics/dashboard    - System analytics");
    println!("   └─ /analytics/tvl-history/:days - TVL history");
    println!("\n   WebSocket:");
    println!("   └─ /ws                     - Real-time updates");
    println!("\n🔍 Monitoring Services:");
    println!("   ├─ Balance Tracker  [ACTIVE]");
    println!("   └─ Vault Monitor    [ACTIVE]");
    println!("\n=================================\n");

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
