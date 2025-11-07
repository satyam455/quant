#!/bin/bash

API="http://localhost:8080"
USER=$(solana address)

echo "🧪 Complete API Test Suite"
echo "=========================="
echo "User: $USER"
echo ""

# Test 1: Initialize
echo "1️⃣  Initialize Vault"
curl -s -X POST $API/register \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\"}" | jq '.'
sleep 2

# Test 2: Get Balance
echo -e "\n2️⃣  Get Balance"
curl -s $API/vault/balance/$USER | jq '.'
sleep 1

# Test 3: Deposit
echo -e "\n3️⃣  Deposit"
curl -s -X POST $API/deposit \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 1000000}" | jq '.'
sleep 2

# Test 4: Get Balance After Deposit
echo -e "\n4️⃣  Get Balance After Deposit"
curl -s $API/vault/balance/$USER | jq '.'
sleep 1

# Test 5: Lock Collateral
echo -e "\n5️⃣  Lock Collateral"
curl -s -X POST $API/lock \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 300000}" | jq '.'
sleep 2

# Test 6: Get Status
echo -e "\n6️⃣  Get Vault Status"
curl -s $API/vault/status/$USER | jq '.'
sleep 1

# Test 7: Get Transactions
echo -e "\n7️⃣  Get Transaction History"
curl -s $API/vault/transactions/$USER | jq '.'
sleep 1

# Test 8: Get TVL
echo -e "\n8️⃣  Get Total Value Locked"
curl -s $API/vault/tvl | jq '.'
sleep 1

# Test 9: Get Alerts
echo -e "\n9️⃣  Get System Alerts"
curl -s $API/vault/alerts | jq '.'

echo -e "\n\n✅ All tests complete!"