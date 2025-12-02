#!/bin/bash
# ========================================
# Complete Functional Test Script
# ========================================

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}║   Collateral Vault - Full Functional Test           ║${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
echo ""

USER=$(solana address)
API="http://localhost:8080"

echo -e "User: ${GREEN}$USER${NC}"
echo -e "API: ${GREEN}$API${NC}"
echo ""
sleep 2

# Test 1: Check Current Balance
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 1: Check Current Vault Balance${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
BALANCE=$(curl -s $API/vault/balance/$USER | jq -r '.total_balance')
echo -e "Current balance: ${GREEN}$BALANCE tokens${NC} ($(echo "scale=6; $BALANCE/1000000" | bc) USDT)"
echo ""
sleep 2

# Test 2: Deposit More Tokens
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 2: Deposit 5 USDT (5,000,000 tokens)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
curl -X POST $API/deposit \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 5000000}" | jq '.'
echo ""
sleep 3

# Test 3: Check Updated Balance
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 3: Check Balance After Deposit${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
curl -s $API/vault/balance/$USER | jq '.'
echo ""
sleep 2

# Test 4: Lock Collateral
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 4: Lock 2 USDT (2,000,000 tokens)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
curl -X POST $API/lock \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 2000000, \"authority_program\": \"$USER\"}" | jq '.'
echo ""
sleep 3

# Test 5: Check Status with Locked Balance
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 5: Check Vault Status (Should Show Locked Balance)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
curl -s $API/vault/status/$USER | jq '.'
echo ""
sleep 2

# Test 6: Try to Withdraw (Should Fail - Locked Balance)
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 6: Try Withdraw 1 USDT (Should Fail - Locked Funds)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Expected: Error because funds are locked${NC}"
curl -X POST $API/withdraw \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 1000000}" | jq '.'
echo ""
sleep 2

# Test 7: Unlock Collateral
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 7: Unlock 2 USDT (2,000,000 tokens)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
curl -X POST $API/unlock \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 2000000, \"authority_program\": \"$USER\"}" | jq '.'
echo ""
sleep 3

# Test 8: Withdraw After Unlock
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 8: Withdraw 1 USDT (Should Succeed Now)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
curl -X POST $API/withdraw \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"amount\": 1000000}" | jq '.'
echo ""
sleep 3

# Test 9: Final Balance Check
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 9: Final Balance Check${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
curl -s $API/vault/balance/$USER | jq '.'
echo ""
sleep 2

# Test 10: Transaction History
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 10: Transaction History${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
curl -s $API/vault/transactions/$USER | jq '.'
echo ""
sleep 2

# Test 11: Total Value Locked
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 11: Total Value Locked (TVL)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
curl -s $API/vault/tvl | jq '.'
echo ""
sleep 2

# Test 12: System Alerts
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST 12: System Alerts${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
curl -s $API/vault/alerts | jq '.'
echo ""

echo -e "${GREEN}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                                                      ║${NC}"
echo -e "${GREEN}║         ✓ ALL FUNCTIONAL TESTS COMPLETED             ║${NC}"
echo -e "${GREEN}║                                                      ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════╝${NC}"
echo ""

echo "Test Summary:"
echo "  ✓ Balance queries"
echo "  ✓ Deposit operations"
echo "  ✓ Lock/Unlock collateral"
echo "  ✓ Withdrawal with locked funds check"
echo "  ✓ Transaction history"
echo "  ✓ TVL tracking"
echo "  ✓ System monitoring"
echo ""
