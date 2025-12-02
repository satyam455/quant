@echo off
REM ========================================
REM API Test Script
REM ========================================

setlocal enabledelayedexpansion

echo.
echo ╔══════════════════════════════════════════════════════╗
echo ║                                                      ║
echo ║   Collateral Vault - API Testing                    ║
echo ║                                                      ║
echo ╚══════════════════════════════════════════════════════╝
echo.

REM Get user's wallet address
for /f "delims=" %%i in ('solana address') do set USER_PUBKEY=%%i

echo User Wallet: %USER_PUBKEY%
echo API Endpoint: http://localhost:8080
echo.
echo ⚠️  Make sure backend server is running (3-start-backend.bat)
echo.
timeout /t 3
echo.

set API=http://localhost:8080

echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo TEST 1: Initialize Vault
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
curl -X POST %API%/register -H "Content-Type: application/json" -d "{\"user_pubkey\": \"%USER_PUBKEY%\", \"authorized_programs\": []}"
echo.
timeout /t 3
echo.

echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo TEST 2: Get Vault Balance
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
curl %API%/vault/balance/%USER_PUBKEY%
echo.
timeout /t 2
echo.

echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo TEST 3: Get Vault Status
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
curl %API%/vault/status/%USER_PUBKEY%
echo.
timeout /t 2
echo.

echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo TEST 4: Get Total Value Locked (TVL)
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
curl %API%/vault/tvl
echo.
timeout /t 2
echo.

echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo TEST 5: Get System Alerts
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
curl %API%/vault/alerts
echo.
timeout /t 2
echo.

echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo TEST 6: Get Transaction History
echo ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
curl %API%/vault/transactions/%USER_PUBKEY%
echo.
echo.

echo ╔══════════════════════════════════════════════════════╗
echo ║                                                      ║
echo ║            ✓ API TESTS COMPLETED                     ║
echo ║                                                      ║
echo ╚══════════════════════════════════════════════════════╝
echo.

echo To test deposit/withdraw operations:
echo   1. Create USDT token account: spl-token create-account YOUR_USDT_MINT
echo   2. Mint tokens: spl-token mint YOUR_USDT_MINT 10000000
echo   3. Then use deposit API endpoint
echo.
pause
