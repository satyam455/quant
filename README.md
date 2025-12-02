# GoQuant Collateral Vault Management System

> A secure, non-custodial collateral vault system for decentralized perpetual futures trading on Solana

##  Quick Links

- **[SETUP.md](SETUP.md)** - Complete deployment and testing commands
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture and technical details
- **[TEST_RESULTS.md](TEST_RESULTS.md)** - Test results and verification

---

##  Overview

The Collateral Vault Management System is the custody layer for GoQuant's decentralized perpetual futures exchange on Solana. It provides:

- **Non-Custodial Storage** - Users maintain full control through PDAs
- **Collateral Management** - Tracks available vs locked collateral
- **Secure Operations** - Atomic deposits, withdrawals, and transfers
- **Real-Time Monitoring** - Balance tracking and security alerts
- **Cross-Program Integration** - CPI support for position managers

### Key Features

 PDA-Based Vaults
 SPL Token Integration
 Lock/Unlock Mechanism
 Real-Time Balance Tracking
 24-Hour Withdrawal Delay (Optional)
 Multi-Signature Support
 Analytics Dashboard

---

##  Quick Start

### Prerequisites

```bash
# Required software
- Rust 1.75+
- Solana CLI 2.0+
- Anchor CLI 0.32.1
- PostgreSQL 14+ (optional)
```

### Installation

```bash
# Configure Solana
solana config set --url https://api.devnet.solana.com
solana-keygen new --outfile ~/.config/solana/id.json
solana airdrop 2

# Create test USDT token
spl-token create-token --decimals 6
# Save the mint address

# Build and deploy
anchor build
anchor deploy

# Build backend
cd back
cargo build --release
cargo run --release
```

**For detailed step-by-step instructions, see [SETUP.md](SETUP.md)**

---

##  API Endpoints

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

**Base URL:** `http://localhost:8080`

---

##  Testing

```bash
# Run smart contract tests
anchor test

# Run backend tests
cd back
cargo test --lib -- --nocapture

# Test API endpoints
export USER=$(solana address)

# Initialize vault
curl -X POST http://localhost:8080/register \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\"}"

# Deposit 1000 USDT
curl -X POST http://localhost:8080/deposit \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 1000000000}" | jq '.'

# Check balance
curl http://localhost:8080/vault/balance/$USER | jq '.'
```

**For complete testing commands, see [SETUP.md](SETUP.md)**

---

##  Security Features

- **PDA-Based Vaults** - Program-controlled addresses
- **Authority Checks** - Owner-only operations
- **Atomic Operations** - All-or-nothing transactions
- **Integer Safety** - Checked arithmetic operations
- **Delayed Withdrawals** - 24-hour security timelock
- **Multi-Signature** - M-of-N signature support
- **Monitoring & Alerts** - Real-time security monitoring

---

##  Common Issues

### Insufficient SOL
```bash
solana airdrop 2
```

### Token Account Not Found
```bash
spl-token create-account YOUR_MINT_ADDRESS
```

### Mint Mismatch
Update `back/src/main.rs` line 38 with correct mint address, then:
```bash
cd back
cargo build --release
cargo run --release
```

### Program ID Mismatch
Update program ID in:
- `programs/collateral_vault/src/lib.rs` (line 11)
- `Anchor.toml` (line 11)
- `back/src/main.rs` (line 31)

Then rebuild:
```bash
anchor clean && anchor build && anchor deploy
```

**For detailed troubleshooting, see [SETUP.md](SETUP.md)**

---

##  Performance

| Metric | Target | Status |
|--------|--------|--------|
| Supported Vaults | 10,000+ |  |
| Deposit Time | < 2s |  |
| Withdrawal Time | < 2s |  |
| Balance Query | < 50ms |  |
| Operations/Second | 100+ |  |

---

##  Project Structure

```
quant/
├── programs/collateral_vault/  # Solana smart contract
├── back/                        # Rust backend service
├── SETUP.md                     # Deployment commands
├── ARCHITECTURE.md              # Technical documentation
└── TEST_RESULTS.md              # Test verification
```

---

##  Documentation

- **[SETUP.md](SETUP.md)** - Complete setup and deployment guide with all commands
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture, data flows, and technical details
- **[TEST_RESULTS.md](TEST_RESULTS.md)** - Test results and system status

---


