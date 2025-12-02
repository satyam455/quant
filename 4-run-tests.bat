@echo off
REM ========================================
REM Test Script for Collateral Vault
REM ========================================

echo.
echo ╔══════════════════════════════════════════════════════╗
echo ║                                                      ║
echo ║   Collateral Vault - Running Tests                  ║
echo ║                                                      ║
echo ╚══════════════════════════════════════════════════════╝
echo.

echo [1/4] Running unit tests...
cd back
call cargo test --lib -- --nocapture
if %errorlevel% neq 0 (
    echo ✗ Unit tests failed!
    cd ..
    pause
    exit /b 1
)
echo ✓ Unit tests passed
echo.

echo [2/4] Running integration tests...
call cargo test --test integration_tests -- --nocapture
if %errorlevel% neq 0 (
    echo ✗ Integration tests failed!
    cd ..
    pause
    exit /b 1
)
echo ✓ Integration tests passed
echo.

cd ..

echo [3/4] Running Anchor tests...
call anchor test
if %errorlevel% neq 0 (
    echo ✗ Anchor tests failed!
    pause
    exit /b 1
)
echo ✓ Anchor tests passed
echo.

echo [4/4] Running clippy checks...
cd back
call cargo clippy -- -D warnings
if %errorlevel% neq 0 (
    echo ⚠️  Clippy found warnings
) else (
    echo ✓ Clippy checks passed
)
cd ..
echo.

echo ╔══════════════════════════════════════════════════════╗
echo ║                                                      ║
echo ║            ✓ ALL TESTS COMPLETED                     ║
echo ║                                                      ║
echo ╚══════════════════════════════════════════════════════╝
echo.
pause
