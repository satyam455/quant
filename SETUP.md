# Collateral Vault - Setup & Deployment Commands

## Prerequisites Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.32.1
avm use 0.32.1

# Install jq for JSON parsing
sudo apt-get update && sudo apt-get install -y jq
```

## Solana Configuration

```bash
# Set cluster to devnet
solana config set --url https://api.devnet.solana.com

# Generate keypair
solana-keygen new --outfile ~/.config/solana/id.json

# Get SOL for gas fees
solana airdrop 2

# Verify setup
solana config get
solana balance
solana address
```

## Create Test USDT Token

```bash
# Create token with 6 decimals
spl-token create-token --decimals 6

# Save the mint address from output
export MINT_ADDRESS="<YOUR_MINT_ADDRESS>"

# Create token account
spl-token create-account $MINT_ADDRESS

# Mint 10,000 USDT (10,000,000,000 tokens with 6 decimals)
spl-token mint $MINT_ADDRESS 10000000000

# Verify balance
spl-token balance $MINT_ADDRESS
```

## Build & Deploy Program

```bash
# Navigate to project
cd ~/Goquant/quant

# Clean and build
anchor clean
anchor build

# Get program ID
solana address -k target/deploy/collateral_vault-keypair.json

# Save program ID
export PROGRAM_ID="<YOUR_PROGRAM_ID>"
```

## Update Configuration Files

### Update Anchor.toml

```bash
nano Anchor.toml
```

Change line 11:
```toml
quant = "YOUR_PROGRAM_ID"
```

Change line 18:
```toml
wallet = "/home/YOUR_USERNAME/.config/solana/id.json"
```

### Update Program lib.rs

```bash
nano programs/collateral_vault/src/lib.rs
```

Change line 11:
```rust
declare_id!("YOUR_PROGRAM_ID");
```

### Update Backend main.rs

```bash
nano back/src/main.rs
```

Change line 31:
```rust
let payer = read_keypair_file("/home/YOUR_USERNAME/.config/solana/id.json")
    .expect("Failed to read Solana keypair");
```

Change line 38:
```rust
let usdt_mint: Pubkey = "YOUR_MINT_ADDRESS"
    .parse()
    .expect("Invalid USDT mint address");
```

## Rebuild & Deploy

```bash
# Rebuild with updated program ID
anchor build

# Deploy to devnet
anchor deploy

# Verify deployment
solana program show $PROGRAM_ID
```

## Build & Run Backend

```bash
# Build backend
cd back
cargo build --release

# Run backend server
cargo run --release
```

## Testing Commands

### Initialize Vault

```bash
export USER=$(solana address)

curl -X POST http://localhost:8080/register \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\"}"
```

### Deposit USDT

```bash
# Deposit 1000 USDT (1,000,000,000 tokens)
curl -X POST http://localhost:8080/deposit \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 1000000000}" | jq '.'
```

### Check Balance

```bash
curl http://localhost:8080/vault/balance/$USER | jq '.'
```

### Lock Collateral

```bash
# Lock 100 USDT
curl -X POST http://localhost:8080/lock \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 100000000, \"authority_program\": \"$USER\"}" | jq '.'
```

### Check Status

```bash
curl http://localhost:8080/vault/status/$USER | jq '.'
```

### Unlock Collateral

```bash
curl -X POST http://localhost:8080/unlock \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 100000000, \"authority_program\": \"$USER\"}" | jq '.'
```

### Withdraw USDT

```bash
# Withdraw 500 USDT
curl -X POST http://localhost:8080/withdraw \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 500000000}" | jq '.'
```

### Check Transaction History

```bash
curl http://localhost:8080/vault/transactions/$USER | jq '.'
```

### Check Total Value Locked

```bash
curl http://localhost:8080/vault/tvl | jq '.'
```

### Check System Alerts

```bash
curl http://localhost:8080/vault/alerts | jq '.'
```

## Run Tests

```bash
# Smart contract tests
cd ~/Goquant/quant
anchor test

# Backend unit tests
cd back
cargo test --lib -- --nocapture

# Integration tests
cargo test --test integration_tests -- --nocapture

# Code quality
cargo clippy -- -D warnings
cargo fmt -- --check
```

## Common Issues

### Insufficient SOL
```bash
solana airdrop 2
```

### Token Account Not Found
```bash
spl-token create-account $MINT_ADDRESS
```

### Program ID Mismatch
Update program ID in:
- `programs/collateral_vault/src/lib.rs` (line 11)
- `Anchor.toml` (line 11)
- `back/src/main.rs` (line 31)

Then rebuild:
```bash
anchor clean && anchor build && anchor deploy
cd back && cargo build --release
```

### Mint Address Mismatch
Update `back/src/main.rs` line 38 with correct mint address, then:
```bash
cd back
cargo build --release
cargo run --release
```
