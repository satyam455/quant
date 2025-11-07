use anchor_client::solana_sdk::signature::{read_keypair_file, Keypair};
use anchor_lang::prelude::Pubkey;
use anyhow::{anyhow, Context, Result};
use std::env;
use std::net::SocketAddr;

pub struct AppConfig {
    pub rpc_url: String,
    pub program_id: Pubkey,
    pub usdt_mint: Pubkey,
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub payer: Keypair,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let rpc_url =
            env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        let payer = if let Ok(secret) = env::var("SOLANA_KEYPAIR") {
            Keypair::from_base58_string(&secret)
        } else {
            let keypair_path = env::var("SOLANA_KEYPAIR_PATH")
                .or_else(|_| env::var("ANCHOR_WALLET"))
                .context("Set SOLANA_KEYPAIR (base58 secret) or SOLANA_KEYPAIR_PATH")?;
            read_keypair_file(&keypair_path)
                .map_err(|err| anyhow!("Failed to read keypair file {}: {}", keypair_path, err))?
        };

        let program_id = env::var("PROGRAM_ID")
            .context("PROGRAM_ID must be set")?
            .parse::<Pubkey>()
            .context("PROGRAM_ID is not a valid pubkey")?;

        let usdt_mint = env::var("USDT_MINT")
            .context("USDT_MINT must be set")?
            .parse::<Pubkey>()
            .context("USDT_MINT is not a valid pubkey")?;

        let database_url =
            env::var("DATABASE_URL").context("DATABASE_URL must be set for persistent storage")?;

        let bind_addr = env::var("BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
            .parse::<SocketAddr>()
            .context("BIND_ADDR must be a valid socket address, e.g. 0.0.0.0:8080")?;

        Ok(Self {
            rpc_url,
            program_id,
            usdt_mint,
            database_url,
            bind_addr,
            payer,
        })
    }
}
