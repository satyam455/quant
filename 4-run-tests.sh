#!/bin/bash
# ========================================
# Test Script for Collateral Vault
# ========================================

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}║   Collateral Vault - Running Tests                  ║${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
echo ""

PASSED=0
FAILED=0

echo -e "${YELLOW}[1/4]${NC} Running unit tests..."
cd back
if cargo test --lib -- --nocapture; then
    echo -e "${GREEN}✓ Unit tests passed${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ Unit tests failed${NC}"
    ((FAILED++))
fi
echo ""

echo -e "${YELLOW}[2/4]${NC} Running integration tests..."
if cargo test --test integration_tests -- --nocapture; then
    echo -e "${GREEN}✓ Integration tests passed${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ Integration tests failed${NC}"
    ((FAILED++))
fi
echo ""

cd ..

echo -e "${YELLOW}[3/4]${NC} Running Anchor tests..."
if anchor test; then
    echo -e "${GREEN}✓ Anchor tests passed${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ Anchor tests failed${NC}"
    ((FAILED++))
fi
echo ""

echo -e "${YELLOW}[4/4]${NC} Running clippy checks..."
cd back
if cargo clippy -- -D warnings 2>&1 | grep -q "warning"; then
    echo -e "${YELLOW}⚠️  Clippy found warnings${NC}"
else
    echo -e "${GREEN}✓ Clippy checks passed${NC}"
    ((PASSED++))
fi
cd ..
echo ""

echo -e "${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}║            TEST SUMMARY                              ║${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    exit 1
fi
