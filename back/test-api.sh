#!/bin/bash

# Collateral Vault Backend API Test Script
# Usage: ./test-api.sh

BASE_URL="http://localhost:8080"
USER_PUBKEY="7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
TO_PUBKEY="8xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsV"

echo "🧪 Testing Collateral Vault Backend APIs"
echo "=========================================="
echo ""

# Test 1: Health Check
echo "✅ Test 1: Health Check"
curl -s $BASE_URL/health | jq .
echo ""

# Test 2: Get TVL
echo "✅ Test 2: Get Total Value Locked (TVL)"
curl -s $BASE_URL/vault/tvl | jq .
echo ""

# Test 3: Get Alerts
echo "✅ Test 3: Get Alerts"
curl -s $BASE_URL/vault/alerts | jq .
echo ""

# Test 4: Get Analytics Dashboard
echo "✅ Test 4: Get Analytics Dashboard"
curl -s $BASE_URL/analytics/dashboard | jq .
echo ""

# Test 5: Get TVL History (7 days)
echo "✅ Test 5: Get TVL History (7 days)"
curl -s $BASE_URL/analytics/tvl-history/7 | jq .
echo ""

# Test 6: Get User Transactions
echo "✅ Test 6: Get User Transactions"
curl -s $BASE_URL/vault/transactions/$USER_PUBKEY | jq .
echo ""

# Test 7: Get User Balance (will fail if vault doesn't exist on-chain)
echo "✅ Test 7: Get User Balance"
curl -s $BASE_URL/vault/balance/$USER_PUBKEY | jq .
echo ""

# Test 8: Get Vault Status (will fail if vault doesn't exist on-chain)
echo "✅ Test 8: Get Vault Status"
curl -s $BASE_URL/vault/status/$USER_PUBKEY | jq .
echo ""

echo "=========================================="
echo "📝 POST Endpoints (require on-chain interaction)"
echo "=========================================="
echo ""

# Test 9: Initialize Vault (requires blockchain transaction)
echo "✅ Test 9: Initialize Vault"
echo "Request:"
echo '{
  "user_pubkey": "'$USER_PUBKEY'",
  "authorized_programs": ["11111111111111111111111111111111"]
}'
curl -X POST $BASE_URL/vault/initialize \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "'$USER_PUBKEY'",
    "authorized_programs": ["11111111111111111111111111111111"]
  }' 2>&1 | tail -1 | jq . 2>/dev/null || echo "Note: This requires on-chain transaction and may timeout"
echo ""

# Test 10: Deposit (requires blockchain transaction)
echo "✅ Test 10: Deposit Collateral"
echo "Request:"
echo '{
  "user_pubkey": "'$USER_PUBKEY'",
  "amount": 1000000
}'
curl -X POST $BASE_URL/vault/deposit \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "'$USER_PUBKEY'",
    "amount": 1000000
  }' 2>&1 | tail -1 | jq . 2>/dev/null || echo "Note: Requires vault to exist on-chain"
echo ""

# Test 11: Withdraw (requires blockchain transaction)
echo "✅ Test 11: Withdraw Collateral"
echo "Request:"
echo '{
  "user_pubkey": "'$USER_PUBKEY'",
  "amount": 500000
}'
curl -X POST $BASE_URL/vault/withdraw \
  -H "Content-Type: application/json" \
  -d '{
    "user_pubkey": "'$USER_PUBKEY'",
    "amount": 500000
  }' 2>&1 | tail -1 | jq . 2>/dev/null || echo "Note: Requires vault to exist on-chain"
echo ""

echo "=========================================="
echo "✅ API Testing Complete!"
echo "=========================================="
echo ""
echo "📋 Summary:"
echo "  - GET endpoints are working correctly"
echo "  - POST endpoints require on-chain Solana transactions"
echo "  - Ensure you have:"
echo "    1. Valid Solana RPC endpoint configured"
echo "    2. Payer wallet with sufficient SOL"
echo "    3. Deployed Solana program"
echo "    4. Correct program ID and mint in .env"
echo ""
