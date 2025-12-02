# Collateral Vault - System Architecture

## Overview

A secure, non-custodial collateral vault system for decentralized perpetual futures trading on Solana.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER INTERFACE                          │
│                    (Web App / Mobile App)                       │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    REST API / WebSocket                         │
│                   (Rust Backend Server)                         │
│  ┌───────────────┬──────────────┬────────────┬────────────────┐ │
│  │ Vault Manager │ Balance      │ CPI        │ Vault Monitor  │ │
│  │               │ Tracker      │ Manager    │                │ │
│  └───────────────┴──────────────┴────────────┴────────────────┘ │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SOLANA BLOCKCHAIN                            │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │        Collateral Vault Program (Anchor)                   │ │
│  │  ┌──────────┬──────────┬──────────┬─────────┬───────────┐  │ │
│  │  │Initialize│  Deposit │ Withdraw │  Lock   │  Unlock   │  │ │
│  │  └──────────┴──────────┴──────────┴─────────┴───────────┘  │ │
│  │  ┌────────────────────────────────────────────────────────┐ │ │
│  │  │           User Vaults (PDA Accounts)                   │ │ │
│  │  │  • Vault State  • Token Accounts  • Balances           │ │ │
│  │  └────────────────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │            SPL Token Program                               │ │
│  │         (USDT Transfer Operations)                         │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    PostgreSQL Database                          │
│  • Transaction History  • Balance Snapshots  • Audit Logs      │
└─────────────────────────────────────────────────────────────────┘
```

## System Components

### 1. Smart Contract (On-Chain)
**Location:** `programs/collateral_vault/`

**Program ID:** Deployed on Solana devnet

**Key Instructions:**
- `initialize_vault` - Creates user vault (PDA)
- `deposit` - Deposits USDT into vault
- `withdraw` - Withdraws USDT from vault
- `lock_collateral` - Locks collateral for positions (CPI only)
- `unlock_collateral` - Unlocks collateral (CPI only)
- `transfer_collateral` - Transfers between vaults
- `initialize_multisig` - Creates multi-sig vault
- `request_withdrawal` - Initiates delayed withdrawal
- `execute_withdrawal` - Completes delayed withdrawal

**Account Structure:**
```rust
pub struct CollateralVault {
    pub owner: Pubkey,
    pub token_account: Pubkey,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub created_at: i64,
    pub bump: u8,
}
```

**PDA Seeds:**
- Vault: `["vault", user_pubkey]`
- Token Account: `["vault_token", user_pubkey]`

### 2. Backend Service (Off-Chain)
**Location:** `back/`

**Tech Stack:**
- Rust + Axum web framework
- Solana RPC client
- PostgreSQL (optional)
- WebSocket support

**Modules:**

**vault_manager.rs**
- Core vault operations
- Transaction building
- RPC communication

**balance_tracker.rs**
- Real-time balance monitoring (30s interval)
- TVL calculation
- Low balance alerts

**cpi_manager.rs**
- Cross-program invocation interface
- Lock/unlock operations
- Batch processing

**vault_monitor.rs**
- Security monitoring
- Health metrics
- Analytics

**handlers.rs**
- HTTP request handlers
- Input validation
- Response formatting

**websocket.rs**
- Real-time updates
- Event broadcasting

**analytics.rs**
- System metrics
- TVL history
- User statistics

**db/postgres.rs**
- Transaction history
- Balance snapshots
- Audit logs

## API Endpoints

### POST Endpoints
- `/register` - Initialize vault
- `/deposit` - Deposit USDT
- `/withdraw` - Withdraw USDT
- `/lock` - Lock collateral
- `/unlock` - Unlock collateral
- `/transfer` - Transfer between vaults

### GET Endpoints
- `/vault/balance/{user}` - Get balance
- `/vault/status/{user}` - Get vault status
- `/vault/transactions/{user}` - Transaction history
- `/vault/tvl` - Total value locked
- `/vault/alerts` - System alerts

### WebSocket
- `ws://localhost:8080/ws` - Real-time updates

## Data Flow

### Deposit Flow
```
User → Backend API → Vault Program
  ↓
SPL Token Transfer (user → vault)
  ↓
Update vault.total_balance
Update vault.available_balance
  ↓
Emit DepositEvent → Backend → Database
```

### Lock Flow (CPI)
```
Position Manager → Vault Program (CPI)
  ↓
Verify authorized program
  ↓
vault.locked_balance += amount
vault.available_balance -= amount
  ↓
Emit CollateralLocked event
```

### Withdraw Flow
```
User → Backend API → Vault Program
  ↓
Check: amount <= vault.available_balance
  ↓
SPL Token Transfer (vault → user) using PDA signature
  ↓
Update balances
  ↓
Emit WithdrawEvent
```

## Security Features

### Access Control
- PDA-based vaults (program-controlled)
- Owner-only deposits/withdrawals
- Authorized programs for lock/unlock
- Multi-signature support

### Safe Math
- All arithmetic uses checked operations
- No overflow/underflow possible
- Balance validation before operations

### Atomic Operations
- SPL Token CPI for transfers
- No partial state updates
- Transaction rollback on failure

### Monitoring
- Real-time balance tracking
- Low balance alerts
- Unauthorized access detection
- Health score calculation

### Advanced Features
- 24-hour withdrawal delay (optional)
- M-of-N multi-signature
- Transaction history audit trail

## Database Schema (PostgreSQL)

### transactions
- id (UUID)
- user (Pubkey)
- tx_type (Deposit/Withdraw/Lock/Unlock)
- amount (u64)
- signature (String)
- status (Pending/Confirmed/Failed)
- timestamp (i64)

### balance_snapshots
- id (UUID)
- user (Pubkey)
- total_balance (u64)
- locked_balance (u64)
- available_balance (u64)
- timestamp (i64)

### alerts
- id (UUID)
- level (Info/Warning/Critical)
- vault (Pubkey)
- message (String)
- timestamp (i64)

## Performance Characteristics

- Deposit/Withdraw: < 2s
- Balance Query: < 50ms
- Lock/Unlock: < 2s
- Supported Vaults: 10,000+
- Operations/Second: 100+

## Network Architecture

```
User Wallet
    ↓
Backend Server (localhost:8080)
    ↓
Solana RPC (api.devnet.solana.com)
    ↓
Solana Blockchain (devnet)
    ↓
Vault Program + SPL Token Program
    ↓
PostgreSQL Database (optional)
```

## Token Standards

- SPL Token Program
- USDT with 6 decimals
- Associated Token Accounts
- PDA token accounts for vaults

## Project Structure

```
quant/
├── programs/collateral_vault/
│   └── src/
│       ├── lib.rs
│       ├── state.rs
│       ├── errors.rs
│       ├── events.rs
│       └── instructions/
│           ├── initialize_vault.rs
│           ├── deposit.rs
│           ├── withdraw.rs
│           ├── lock.rs
│           ├── unlock.rs
│           ├── transfer_collateral.rs
│           ├── multisig.rs
│           └── security.rs
│
└── back/
    └── src/
        ├── main.rs
        ├── vault_manager.rs
        ├── handlers.rs
        ├── balance_tracker.rs
        ├── cpi_manager.rs
        ├── vault_monitor.rs
        ├── analytics.rs
        ├── websocket.rs
        └── db/
            ├── mod.rs
            └── postgres.rs
```

## Technology Stack

### On-Chain
- Anchor Framework 0.32.1
- Solana SDK 2.0+
- SPL Token Program

### Off-Chain
- Rust 1.75+
- Axum (HTTP server)
- Tokio (async runtime)
- Solana RPC Client
- PostgreSQL 14+
- WebSocket

## Deployment Environment

- **Network:** Solana Devnet
- **RPC:** https://api.devnet.solana.com
- **Backend:** localhost:8080
- **Database:** localhost:5432 (optional)
