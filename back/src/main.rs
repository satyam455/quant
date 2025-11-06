use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use axum::{routing::post, Extension, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

mod handlers;
mod vault_manager;

fn main() -> Result<()> {
    let payer = read_keypair_file("/home/satyam/.config/solana/id.json")
        .expect("Failed to read Solana keypair");

    let program_id: Pubkey = "4NYik8PfZkQdj89AjVxX8LWHZyPKWQ647XKNpCMk6gAR"
        .parse()
        .unwrap();

    // For devnet, you can use a test USDT mint or create your own
    // Example devnet USDT: Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr
    let usdt_mint: Pubkey = "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"
        .parse()
        .expect("Invalid USDT mint address");

    let vault_mgr = vault_manager::VaultManager::new(
        "https://api.devnet.solana.com".to_string(),
        payer,
        program_id,
        usdt_mint,
    )
    .expect("❌ Failed to create VaultManager");

    println!("✅ VaultManager initialized successfully");
    println!("📍 Program ID: {}", program_id);
    println!("💵 USDT Mint: {}", usdt_mint);

    let vault_mgr = Arc::new(vault_mgr);

    start_server(vault_mgr)
}

#[tokio::main]
async fn start_server(vault_mgr: Arc<vault_manager::VaultManager>) -> Result<()> {
    run_server(vault_mgr).await
}

async fn run_server(vault_mgr: Arc<vault_manager::VaultManager>) -> Result<()> {
    let app = Router::new()
        .route("/register", post(handlers::register_vault))
        .route("/deposit", post(handlers::deposit))
        .route("/withdraw", post(handlers::withdraw))
        .route("/lock", post(handlers::lock))
        .route("/unlock", post(handlers::unlock))
        .route("/transfer", post(handlers::transfer))
        .layer(Extension(vault_mgr));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;
    println!("✅ Vault backend running at http://{}", addr);
    println!("📡 Endpoints:");
    println!("   POST /register  - Initialize vault");
    println!("   POST /deposit   - Deposit collateral");
    println!("   POST /withdraw  - Withdraw collateral");
    println!("   POST /lock      - Lock collateral");
    println!("   POST /unlock    - Unlock collateral");
    println!("   POST /transfer  - Transfer collateral");

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
