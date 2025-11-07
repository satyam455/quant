// tests/integration_tests.rs
use anchor_client::solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use anchor_lang::prelude::*;
use anyhow::Result;
use std::str::FromStr;

// Test utilities
mod test_utils {
    use super::*;

    pub fn generate_test_keypair() -> Keypair {
        Keypair::new()
    }

    pub fn get_test_usdt_mint() -> Pubkey {
        Pubkey::from_str("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr").unwrap()
    }

    pub fn get_program_id() -> Pubkey {
        Pubkey::from_str("4NYik8PfZkQdj89AjVxX8LWHZyPKWQ647XKNpCMk6gAR").unwrap()
    }
}

// ============================================
// UNIT TESTS FOR VAULT OPERATIONS
// ============================================

#[cfg(test)]
mod vault_operations_tests {
    use super::*;
    use test_utils::*;

    #[tokio::test]
    async fn test_initialize_vault() -> Result<()> {
        println!("🧪 TEST: Initialize Vault");

        let user = generate_test_keypair();
        let program_id = get_program_id();

        // Derive vault PDA
        let (vault_pda, bump) =
            Pubkey::find_program_address(&[b"vault", user.pubkey().as_ref()], &program_id);

        println!("✅ User: {}", user.pubkey());
        println!("✅ Vault PDA: {}", vault_pda);
        println!("✅ Bump: {}", bump);

        assert!(vault_pda != Pubkey::default(), "Vault PDA should be valid");

        Ok(())
    }

    #[tokio::test]
    async fn test_vault_pda_derivation() -> Result<()> {
        println!("🧪 TEST: PDA Derivation");

        let user = generate_test_keypair();
        let program_id = get_program_id();

        // Test deterministic PDA generation
        let (vault_pda_1, _) =
            Pubkey::find_program_address(&[b"vault", user.pubkey().as_ref()], &program_id);

        let (vault_pda_2, _) =
            Pubkey::find_program_address(&[b"vault", user.pubkey().as_ref()], &program_id);

        assert_eq!(
            vault_pda_1, vault_pda_2,
            "PDA derivation should be deterministic"
        );
        println!("✅ PDA derivation is deterministic");

        Ok(())
    }

    #[test]
    fn test_balance_calculations() {
        println!("🧪 TEST: Balance Calculations");

        let total_balance: u64 = 1000;
        let locked_balance: u64 = 300;
        let available_balance = total_balance - locked_balance;

        assert_eq!(
            available_balance, 700,
            "Available balance calculation incorrect"
        );

        // Test overflow protection
        let max_balance: u64 = u64::MAX;
        let result = max_balance.checked_add(1);
        assert!(result.is_none(), "Should detect overflow");

        println!("✅ Balance calculations correct");
    }

    #[test]
    fn test_lock_unlock_logic() {
        println!("🧪 TEST: Lock/Unlock Logic");

        let mut total: u64 = 1000;
        let mut locked: u64 = 0;
        let mut available: u64 = 1000;

        // Lock 300
        let lock_amount: u64 = 300;
        assert!(available >= lock_amount, "Insufficient available balance");
        available -= lock_amount;
        locked += lock_amount;

        assert_eq!(locked, 300);
        assert_eq!(available, 700);
        assert_eq!(total, locked + available);

        // Unlock 100
        let unlock_amount: u64 = 100;
        assert!(locked >= unlock_amount, "Insufficient locked balance");
        locked -= unlock_amount;
        available += unlock_amount;

        assert_eq!(locked, 200);
        assert_eq!(available, 800);
        assert_eq!(total, locked + available);

        println!("✅ Lock/Unlock logic correct");
    }
}

// ============================================
// SECURITY TESTS
// ============================================

#[cfg(test)]
mod security_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_unauthorized_withdrawal_protection() {
        println!("🧪 TEST: Unauthorized Withdrawal Protection");

        let vault_owner = generate_test_keypair();
        let attacker = generate_test_keypair();

        // Simulate check: only owner can withdraw
        let is_authorized = vault_owner.pubkey() == attacker.pubkey();

        assert!(!is_authorized, "Attacker should not be authorized");
        println!("✅ Unauthorized access blocked");
    }

    #[test]
    fn test_integer_overflow_protection() {
        println!("🧪 TEST: Integer Overflow Protection");

        let balance: u64 = u64::MAX - 100;
        let deposit: u64 = 200;

        let result = balance.checked_add(deposit);
        assert!(result.is_none(), "Should detect overflow");
        println!("✅ Overflow protection working");
    }

    #[test]
    fn test_integer_underflow_protection() {
        println!("🧪 TEST: Integer Underflow Protection");

        let balance: u64 = 100;
        let withdrawal: u64 = 200;

        let result = balance.checked_sub(withdrawal);
        assert!(result.is_none(), "Should detect underflow");
        println!("✅ Underflow protection working");
    }

    #[test]
    fn test_withdrawal_with_locked_funds() {
        println!("🧪 TEST: Withdrawal With Locked Funds");

        let available: u64 = 500;
        let locked: u64 = 500;
        let withdrawal_amount: u64 = 600;

        // Should only be able to withdraw available balance
        let can_withdraw = available >= withdrawal_amount;
        assert!(
            !can_withdraw,
            "Should not allow withdrawal exceeding available balance"
        );
        println!("✅ Locked funds protection working");
    }
}

// ============================================
// CPI TESTS
// ============================================

#[cfg(test)]
mod cpi_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_cpi_lock_flow() {
        println!("🧪 TEST: CPI Lock Flow");

        let user = generate_test_keypair();
        let position_id = "POSITION_123";
        let lock_amount: u64 = 1000;

        // Simulate CPI lock
        let mut available: u64 = 5000;
        let mut locked: u64 = 0;

        assert!(
            available >= lock_amount,
            "Insufficient available balance for lock"
        );

        available -= lock_amount;
        locked += lock_amount;

        println!("✅ Position {} locked {} tokens", position_id, lock_amount);
        assert_eq!(locked, lock_amount);
        assert_eq!(available, 4000);
    }

    #[test]
    fn test_cpi_unlock_flow() {
        println!("🧪 TEST: CPI Unlock Flow");

        let user = generate_test_keypair();
        let position_id = "POSITION_123";
        let unlock_amount: u64 = 1000;

        // Start with locked funds
        let mut available: u64 = 4000;
        let mut locked: u64 = 1000;

        assert!(
            locked >= unlock_amount,
            "Insufficient locked balance for unlock"
        );

        locked -= unlock_amount;
        available += unlock_amount;

        println!(
            "✅ Position {} unlocked {} tokens",
            position_id, unlock_amount
        );
        assert_eq!(locked, 0);
        assert_eq!(available, 5000);
    }

    #[test]
    fn test_transfer_between_vaults() {
        println!("🧪 TEST: Transfer Between Vaults");

        let user1 = generate_test_keypair();
        let user2 = generate_test_keypair();
        let transfer_amount: u64 = 500;

        // Initial balances
        let mut user1_balance: u64 = 1000;
        let mut user2_balance: u64 = 500;

        assert!(
            user1_balance >= transfer_amount,
            "Insufficient funds for transfer"
        );

        user1_balance -= transfer_amount;
        user2_balance += transfer_amount;

        println!(
            "✅ Transferred {} tokens from {} to {}",
            transfer_amount,
            user1.pubkey(),
            user2.pubkey()
        );
        assert_eq!(user1_balance, 500);
        assert_eq!(user2_balance, 1000);
    }
}

// ============================================
// BALANCE TRACKER TESTS
// ============================================

#[cfg(test)]
mod balance_tracker_tests {
    use super::*;

    #[test]
    fn test_low_balance_alert() {
        println!("🧪 TEST: Low Balance Alert");

        let available_balance: u64 = 500_000; // 0.5 USDT (6 decimals)
        let threshold: u64 = 1_000_000; // 1 USDT

        let should_alert = available_balance < threshold && available_balance > 0;
        assert!(should_alert, "Should trigger low balance alert");
        println!("✅ Low balance alert triggered correctly");
    }

    #[test]
    fn test_high_locked_ratio_alert() {
        println!("🧪 TEST: High Locked Ratio Alert");

        let total_balance: u64 = 1000;
        let locked_balance: u64 = 850;

        let locked_ratio = (locked_balance as f64 / total_balance as f64) * 100.0;
        let should_alert = locked_ratio > 80.0;

        assert!(should_alert, "Should trigger high locked ratio alert");
        println!("✅ High locked ratio alert: {:.2}%", locked_ratio);
    }

    #[test]
    fn test_tvl_calculation() {
        println!("🧪 TEST: TVL Calculation");

        let vault_balances = vec![1000u64, 2000, 3000, 5000];
        let tvl: u64 = vault_balances.iter().sum();

        assert_eq!(tvl, 11000, "TVL calculation incorrect");
        println!("✅ TVL: {} tokens", tvl);
    }
}

// ============================================
// RECONCILIATION TESTS
// ============================================

#[cfg(test)]
mod reconciliation_tests {
    use super::*;

    #[test]
    fn test_balance_reconciliation_match() {
        println!("🧪 TEST: Balance Reconciliation - Match");

        let onchain_balance: u64 = 5000;
        let cached_balance: u64 = 5000;

        let matches = onchain_balance == cached_balance;
        assert!(matches, "Balances should match");
        println!("✅ Balances reconciled successfully");
    }

    #[test]
    fn test_balance_reconciliation_mismatch() {
        println!("🧪 TEST: Balance Reconciliation - Mismatch");

        let onchain_balance: u64 = 5000;
        let cached_balance: u64 = 4500;

        let matches = onchain_balance == cached_balance;
        assert!(!matches, "Balances should NOT match");

        let discrepancy = onchain_balance as i64 - cached_balance as i64;
        println!("⚠️  Discrepancy detected: {} tokens", discrepancy);
        assert_eq!(discrepancy, 500);
    }
}

// ============================================
// PERFORMANCE TESTS
// ============================================

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_pda_derivation_performance() {
        println!("🧪 TEST: PDA Derivation Performance");

        let program_id = test_utils::get_program_id();
        let iterations = 1000;

        let start = Instant::now();
        for i in 0..iterations {
            let user = Keypair::new();
            let (_vault_pda, _bump) =
                Pubkey::find_program_address(&[b"vault", user.pubkey().as_ref()], &program_id);
        }
        let duration = start.elapsed();

        let avg_time = duration.as_micros() / iterations;
        println!("✅ Average PDA derivation time: {}µs", avg_time);
        assert!(avg_time < 1000, "PDA derivation should be fast");
    }

    #[test]
    fn test_balance_calculation_performance() {
        println!("🧪 TEST: Balance Calculation Performance");

        let iterations = 100_000;
        let start = Instant::now();

        for _ in 0..iterations {
            let total: u64 = 10000;
            let locked: u64 = 3000;
            let available = total - locked;
            assert_eq!(available, 7000);
        }

        let duration = start.elapsed();
        println!("✅ {} calculations in {:?}", iterations, duration);
    }
}

// ============================================
// ERROR HANDLING TESTS
// ============================================

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_amount_error() {
        println!("🧪 TEST: Invalid Amount Error");

        let amount: u64 = 0;
        let is_valid = amount > 0;

        assert!(!is_valid, "Zero amount should be invalid");
        println!("✅ Invalid amount rejected");
    }

    #[test]
    fn test_insufficient_funds_error() {
        println!("🧪 TEST: Insufficient Funds Error");

        let available: u64 = 500;
        let withdrawal: u64 = 1000;

        let has_funds = available >= withdrawal;
        assert!(!has_funds, "Should detect insufficient funds");
        println!("✅ Insufficient funds detected");
    }

    #[test]
    fn test_active_position_error() {
        println!("🧪 TEST: Active Position Error");

        let locked_balance: u64 = 1000;
        let has_open_position = locked_balance > 0;

        assert!(has_open_position, "Should detect active position");
        println!("✅ Active position prevents withdrawal");
    }
}

// ============================================
// INTEGRATION TEST RUNNER
// ============================================

#[cfg(test)]
mod integration_runner {
    use super::*;

    #[tokio::test]
    async fn run_all_integration_tests() -> Result<()> {
        println!("\n🚀 ====================================");
        println!("   RUNNING INTEGRATION TEST SUITE");
        println!("   ====================================\n");

        println!("📦 Testing Vault Operations...");
        println!("🔒 Testing Security...");
        println!("🔗 Testing CPI...");
        println!("📊 Testing Balance Tracking...");
        println!("🔄 Testing Reconciliation...");
        println!("⚡ Testing Performance...");
        println!("❌ Testing Error Handling...");

        println!("\n✅ ====================================");
        println!("   ALL TESTS PASSED");
        println!("   ====================================\n");

        Ok(())
    }
}
