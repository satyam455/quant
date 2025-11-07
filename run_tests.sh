#!/bin/bash

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}║   Collateral Vault Management System                ║${NC}"
echo -e "${BLUE}║   Comprehensive Test Suite                          ║${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to print section headers
print_section() {
    echo ""
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}  $1${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# Function to check command success
check_status() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ $1 passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $1 failed${NC}"
        return 1
    fi
}

# Initialize test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# ============================================
# 1. ENVIRONMENT CHECKS
# ============================================
print_section "1. Environment Checks"

echo "Checking Rust installation..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo -e "${GREEN}✓ Rust: $RUST_VERSION${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}✗ Rust not found${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

echo "Checking Anchor installation..."
if command -v anchor &> /dev/null; then
    ANCHOR_VERSION=$(anchor --version)
    echo -e "${GREEN}✓ Anchor: $ANCHOR_VERSION${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}✗ Anchor not found${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

echo "Checking Solana CLI..."
if command -v solana &> /dev/null; then
    SOLANA_VERSION=$(solana --version)
    echo -e "${GREEN}✓ Solana: $SOLANA_VERSION${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}✗ Solana CLI not found${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# ============================================
# 2. BUILD TESTS
# ============================================
print_section "2. Build Tests"

echo "Building Anchor program..."
cd programs/collateral_vault
if anchor build > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Anchor program build successful${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}✗ Anchor program build failed${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
cd ../..

echo "Building backend service..."
cd back
if cargo build --release > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Backend build successful${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}✗ Backend build failed${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
cd ..

# ============================================
# 3. UNIT TESTS
# ============================================
print_section "3. Unit Tests"

echo "Running Rust backend unit tests..."
cd back
if cargo test --lib -- --nocapture; then
    check_status "Unit tests"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    check_status "Unit tests"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
cd ..

# ============================================
# 4. INTEGRATION TESTS
# ============================================
print_section "4. Integration Tests"

echo "Running integration tests..."
cd back
if cargo test --test integration_tests -- --nocapture; then
    check_status "Integration tests"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    check_status "Integration tests"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
cd ..

# ============================================
# 5. ANCHOR PROGRAM TESTS
# ============================================
print_section "5. Anchor Program Tests"

echo "Running Anchor tests..."
if anchor test; then
    check_status "Anchor program tests"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    check_status "Anchor program tests"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# ============================================
# 6. SECURITY TESTS
# ============================================
print_section "6. Security Tests"

echo "Running security checks with cargo-audit..."
cd back
if command -v cargo-audit &> /dev/null; then
    if cargo audit; then
        check_status "Security audit"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        check_status "Security audit"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
else
    echo -e "${YELLOW}⚠ cargo-audit not installed, skipping${NC}"
fi
cd ..

# ============================================
# 7. CODE COVERAGE
# ============================================
print_section "7. Code Coverage"

echo "Generating code coverage report..."
cd back
if command -v cargo-tarpaulin &> /dev/null; then
    if cargo tarpaulin --out Html --output-dir coverage; then
        echo -e "${GREEN}✓ Coverage report generated in back/coverage/${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ Coverage generation failed${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
else
    echo -e "${YELLOW}⚠ cargo-tarpaulin not installed, skipping coverage${NC}"
fi
cd ..

# ============================================
# 8. LINTING & FORMATTING
# ============================================
print_section "8. Linting & Formatting"

echo "Running cargo clippy..."
cd back
if cargo clippy -- -D warnings > /dev/null 2>&1; then
    check_status "Clippy linting"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    check_status "Clippy linting"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

echo "Checking code formatting..."
if cargo fmt -- --check > /dev/null 2>&1; then
    check_status "Code formatting"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${YELLOW}⚠ Code not formatted, run 'cargo fmt'${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
cd ..

# ============================================
# 9. PERFORMANCE TESTS
# ============================================
print_section "9. Performance Tests"

echo "Running performance benchmarks..."
cd back
if cargo test --release performance -- --nocapture; then
    check_status "Performance tests"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    check_status "Performance tests"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))
cd ..

# ============================================
# TEST SUMMARY
# ============================================
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}║                   TEST SUMMARY                       ║${NC}"
echo -e "${BLUE}║                                                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════╝${NC}"
echo ""

echo -e "Total Tests:  ${BLUE}$TOTAL_TESTS${NC}"
echo -e "Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed:       ${RED}$FAILED_TESTS${NC}"

PASS_RATE=$(awk "BEGIN {printf \"%.2f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")
echo -e "Pass Rate:    ${YELLOW}${PASS_RATE}%${NC}"

echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}╔══════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                      ║${NC}"
    echo -e "${GREEN}║            ✓ ALL TESTS PASSED!                       ║${NC}"
    echo -e "${GREEN}║                                                      ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔══════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                                                      ║${NC}"
    echo -e "${RED}║            ✗ SOME TESTS FAILED                       ║${NC}"
    echo -e "${RED}║                                                      ║${NC}"
    echo -e "${RED}╚══════════════════════════════════════════════════════╝${NC}"
    exit 1
fi