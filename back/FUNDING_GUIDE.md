# USDT Funding Guide - Which Address to Fund?

## Overview
To test the collateral vault system, you need to fund specific addresses with USDT tokens.

---

## 🎯 **WHICH ADDRESS NEEDS USDT?**

### User Wallet Address (For Testing)
**Address:** `25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x`

This is your payer wallet that will be used as the test user for:
- Initializing vaults
- Depositing collateral
- Withdrawing collateral
- All vault operations

**Current Status:**
- ✅ Has 70.102298472 SOL (sufficient for transaction fees)
- ❌ Needs USDT tokens for testing deposits

---

## 📦 **USDT Mint Information**

**USDT Mint Address:** `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`

**Network:** Solana Devnet

---

## 🔧 **Steps to Fund the Wallet**

### Step 1: Create Associated Token Account for USDT

```bash
spl-token create-account 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU \
  --owner 25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x \
  --url devnet
```

This will create an associated token account (ATA) for the user wallet to hold USDT tokens.

**Expected Output:**
```
Creating account...
Account: <ATA-ADDRESS>
```

---

### Step 2: Get USDT Tokens

#### Option A: If you control the USDT mint authority

```bash
# Mint USDT tokens to the user's token account
spl-token mint 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU \
  1000000000 \
  <USER-ATA-ADDRESS> \
  --url devnet
```

#### Option B: If you need to create a test USDT mint

```bash
# Create a new test USDT mint
spl-token create-token --decimals 6 --url devnet

# Mint some tokens
spl-token mint <NEW-MINT-ADDRESS> 1000000000 --url devnet

# Update .env file with new mint address
# USDT_MINT=<NEW-MINT-ADDRESS>
```

---

### Step 3: Verify Token Balance

```bash
spl-token balance 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU \
  --owner 25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x \
  --url devnet
```

**Expected Output:**
```
1000000000
```

---

## 📊 **Required Balances for Testing**

| Asset | Address | Current | Required | Status |
|-------|---------|---------|----------|--------|
| SOL | 25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x | 70.1 SOL | 1+ SOL | ✅ Ready |
| USDT | ATA of above address | 0 | 1,000,000+ | ❌ Need to fund |

---

## 🧪 **Testing Scenarios**

Once funded, you can test these operations:

### 1. Initialize Vault (Requires: SOL for fees)
```bash
curl -X POST http://localhost:8080/vault/initialize \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x"
  }'
```

### 2. Deposit Collateral (Requires: USDT tokens)
```bash
curl -X POST http://localhost:8080/vault/deposit \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x",
    "amount": 1000000
  }'
```

### 3. Check Balance (After deposit)
```bash
curl http://localhost:8080/vault/balance/25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x
```

### 4. Withdraw Collateral
```bash
curl -X POST http://localhost:8080/vault/withdraw \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x",
    "amount": 500000
  }'
```

---

## 🔍 **Quick Check Commands**

### Check SOL Balance
```bash
solana balance 25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x --url devnet
```

### Check USDT Balance
```bash
spl-token accounts 25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x --url devnet
```

### Check Token Account
```bash
spl-token account-info 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU \
  --owner 25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x \
  --url devnet
```

---

## ⚠️ **Common Issues**

### Issue 1: "Account not found" error
**Solution:** Create the associated token account first (Step 1)

### Issue 2: "Insufficient balance" error
**Solution:** Mint more USDT tokens to the user's ATA (Step 2)

### Issue 3: "Invalid mint" error
**Solution:** Verify the USDT mint address in .env matches the actual mint

---

## 📝 **Summary**

**TO FUND:**
1. **Main Address:** `25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x`
2. **With:** USDT tokens (mint: `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`)
3. **Amount:** 1,000,000+ USDT (1 USDT = 1,000,000 smallest units with 6 decimals)

**Steps:**
1. Create ATA for USDT
2. Mint/Transfer USDT to the ATA
3. Verify balance
4. Start testing APIs

---

## 🚀 **Ready to Test?**

Once you've funded the address with USDT, run:

```bash
# Check everything is ready
curl http://localhost:8080/health

# Initialize vault
curl -X POST http://localhost:8080/vault/initialize \
  -H "Content-Type: application/json" \
  -d '{"user_pubkey":"25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x"}'

# Deposit 1 USDT
curl -X POST http://localhost:8080/vault/deposit \
  -H "Content-Type: application/json" \
  -d '{"user_pubkey":"25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x","amount":1000000}'

# Check balance
curl http://localhost:8080/vault/balance/25vpiwU2DJJiCV9gpBDuJEKAEs16uXH6ZY8CXX5VV55x
```

---

**Need Help?**
- Check server logs: `tail -f /tmp/backend.log`
- Check Solana explorer: https://explorer.solana.com/?cluster=devnet
- API Documentation: See `API_TEST_RESULTS.md`
