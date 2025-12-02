#!/bin/bash
# ========================================
# Add Authorized Program to Vault
# ========================================

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}║   Add Authorized Program to Vault                    ║${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
echo ""

USER=$(solana address)
API="http://localhost:8080"

echo -e "User: ${GREEN}$USER${NC}"
echo ""

# For testing, we'll authorize the program itself or the user's wallet
# In production, you'd authorize specific position management programs

echo -e "${YELLOW}Adding program to authorized list...${NC}"
echo ""

# Add the vault program itself as authorized (for testing)
PROGRAM_ID="GfHdK9T6kBwS55D9pv97CbNE9PdP4kpASxMipM7gWSKa"

curl -X POST $API/add-authorized-program \
  -H "Content-Type: application/json" \
  -d "{\"user_pubkey\": \"$USER\", \"program_pubkey\": \"$PROGRAM_ID\"}" | jq '.'

echo ""
echo -e "${GREEN}✓ Program authorized${NC}"
echo ""

echo "You can now lock/unlock collateral for testing."
echo ""
