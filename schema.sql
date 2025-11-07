-- Database Schema for Collateral Vault Management System
-- PostgreSQL 14+

-- ============================================
-- 1. VAULT ACCOUNTS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS vault_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_pubkey VARCHAR(44) NOT NULL UNIQUE,
    vault_pda VARCHAR(44) NOT NULL UNIQUE,
    token_account VARCHAR(44) NOT NULL,
    total_balance BIGINT NOT NULL DEFAULT 0,
    locked_balance BIGINT NOT NULL DEFAULT 0,
    available_balance BIGINT NOT NULL DEFAULT 0,
    total_deposited BIGINT NOT NULL DEFAULT 0,
    total_withdrawn BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE', -- ACTIVE, FROZEN, CLOSED
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_balance CHECK (total_balance >= 0),
    CONSTRAINT chk_locked CHECK (locked_balance >= 0),
    CONSTRAINT chk_available CHECK (available_balance >= 0),
    CONSTRAINT chk_balance_sum CHECK (total_balance = locked_balance + available_balance)
);

CREATE INDEX idx_vault_owner ON vault_accounts(owner_pubkey);
CREATE INDEX idx_vault_status ON vault_accounts(status);
CREATE INDEX idx_vault_created ON vault_accounts(created_at DESC);

-- ============================================
-- 2. TRANSACTION HISTORY TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES vault_accounts(id) ON DELETE CASCADE,
    user_pubkey VARCHAR(44) NOT NULL,
    tx_type VARCHAR(20) NOT NULL, -- INITIALIZE, DEPOSIT, WITHDRAW, LOCK, UNLOCK, TRANSFER
    amount BIGINT NOT NULL,
    signature VARCHAR(88) NOT NULL UNIQUE,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING', -- PENDING, CONFIRMED, FAILED
    error_message TEXT,
    slot BIGINT,
    block_time BIGINT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMP,
    
    CONSTRAINT chk_tx_amount CHECK (amount >= 0)
);

CREATE INDEX idx_tx_vault ON transactions(vault_id);
CREATE INDEX idx_tx_user ON transactions(user_pubkey);
CREATE INDEX idx_tx_type ON transactions(tx_type);
CREATE INDEX idx_tx_status ON transactions(status);
CREATE INDEX idx_tx_signature ON transactions(signature);
CREATE INDEX idx_tx_created ON transactions(created_at DESC);

-- ============================================
-- 3. BALANCE SNAPSHOTS TABLE (Hourly/Daily)
-- ============================================
CREATE TABLE IF NOT EXISTS balance_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES vault_accounts(id) ON DELETE CASCADE,
    user_pubkey VARCHAR(44) NOT NULL,
    total_balance BIGINT NOT NULL,
    locked_balance BIGINT NOT NULL,
    available_balance BIGINT NOT NULL,
    snapshot_type VARCHAR(10) NOT NULL, -- HOURLY, DAILY
    snapshot_time TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_snapshot_balance CHECK (total_balance >= 0),
    CONSTRAINT unique_snapshot UNIQUE (vault_id, snapshot_type, snapshot_time)
);

CREATE INDEX idx_snapshot_vault ON balance_snapshots(vault_id);
CREATE INDEX idx_snapshot_time ON balance_snapshots(snapshot_time DESC);
CREATE INDEX idx_snapshot_type ON balance_snapshots(snapshot_type);

-- ============================================
-- 4. RECONCILIATION LOGS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS reconciliation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES vault_accounts(id) ON DELETE CASCADE,
    user_pubkey VARCHAR(44) NOT NULL,
    onchain_balance BIGINT NOT NULL,
    offchain_balance BIGINT NOT NULL,
    discrepancy BIGINT NOT NULL, -- onchain - offchain
    status VARCHAR(20) NOT NULL, -- MATCH, MISMATCH, RESOLVED
    resolution_action TEXT,
    checked_at TIMESTAMP NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMP
);

CREATE INDEX idx_reconcile_vault ON reconciliation_logs(vault_id);
CREATE INDEX idx_reconcile_status ON reconciliation_logs(status);
CREATE INDEX idx_reconcile_checked ON reconciliation_logs(checked_at DESC);

-- ============================================
-- 5. AUDIT TRAIL TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_pubkey VARCHAR(44) NOT NULL,
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(30) NOT NULL, -- VAULT, TRANSACTION, BALANCE
    resource_id VARCHAR(100),
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_user ON audit_logs(user_pubkey);
CREATE INDEX idx_audit_action ON audit_logs(action);
CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_details ON audit_logs USING gin(details);

-- ============================================
-- 6. SYSTEM METRICS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS system_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_type VARCHAR(50) NOT NULL, -- TVL, ACTIVE_VAULTS, TPS, etc.
    metric_value BIGINT NOT NULL,
    metadata JSONB,
    recorded_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_metrics_type ON system_metrics(metric_type);
CREATE INDEX idx_metrics_recorded ON system_metrics(recorded_at DESC);

-- ============================================
-- 7. ALERTS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type VARCHAR(30) NOT NULL, -- LOW_BALANCE, HIGH_LOCKED_RATIO, UNAUTHORIZED_ACCESS, DISCREPANCY
    severity VARCHAR(20) NOT NULL, -- LOW, MEDIUM, HIGH, CRITICAL
    vault_id UUID REFERENCES vault_accounts(id) ON DELETE CASCADE,
    user_pubkey VARCHAR(44),
    message TEXT NOT NULL,
    metadata JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE', -- ACTIVE, ACKNOWLEDGED, RESOLVED
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMP,
    resolved_at TIMESTAMP
);

CREATE INDEX idx_alerts_type ON alerts(alert_type);
CREATE INDEX idx_alerts_severity ON alerts(severity);
CREATE INDEX idx_alerts_status ON alerts(status);
CREATE INDEX idx_alerts_created ON alerts(created_at DESC);

-- ============================================
-- 8. CPI OPERATIONS LOG TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS cpi_operations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation_type VARCHAR(30) NOT NULL, -- LOCK, UNLOCK, TRANSFER
    source_vault_id UUID REFERENCES vault_accounts(id),
    target_vault_id UUID REFERENCES vault_accounts(id),
    amount BIGINT NOT NULL,
    position_id VARCHAR(100),
    caller_program VARCHAR(44),
    signature VARCHAR(88) NOT NULL,
    status VARCHAR(20) NOT NULL, -- SUCCESS, FAILED
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cpi_source ON cpi_operations(source_vault_id);
CREATE INDEX idx_cpi_target ON cpi_operations(target_vault_id);
CREATE INDEX idx_cpi_type ON cpi_operations(operation_type);
CREATE INDEX idx_cpi_created ON cpi_operations(created_at DESC);

-- ============================================
-- TRIGGERS FOR AUTOMATIC UPDATES
-- ============================================

-- Update vault_accounts.updated_at on changes
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_vault_updated_at
BEFORE UPDATE ON vault_accounts
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- VIEWS FOR ANALYTICS
-- ============================================

-- View: Active vaults with current balances
CREATE OR REPLACE VIEW active_vaults AS
SELECT 
    id,
    owner_pubkey,
    vault_pda,
    total_balance,
    locked_balance,
    available_balance,
    (locked_balance::DECIMAL / NULLIF(total_balance, 0) * 100) as locked_ratio,
    total_deposited,
    total_withdrawn,
    created_at
FROM vault_accounts
WHERE status = 'ACTIVE';

-- View: TVL summary
CREATE OR REPLACE VIEW tvl_summary AS
SELECT 
    COUNT(*) as total_vaults,
    SUM(total_balance) as total_value_locked,
    SUM(locked_balance) as total_locked,
    SUM(available_balance) as total_available,
    AVG(total_balance) as avg_balance,
    NOW() as calculated_at
FROM vault_accounts
WHERE status = 'ACTIVE';

-- View: Recent transactions
CREATE OR REPLACE VIEW recent_transactions AS
SELECT 
    t.id,
    t.user_pubkey,
    va.vault_pda,
    t.tx_type,
    t.amount,
    t.signature,
    t.status,
    t.created_at,
    t.confirmed_at
FROM transactions t
JOIN vault_accounts va ON t.vault_id = va.id
ORDER BY t.created_at DESC
LIMIT 100;

-- ============================================
-- FUNCTIONS FOR COMMON OPERATIONS
-- ============================================

-- Function: Get vault statistics
CREATE OR REPLACE FUNCTION get_vault_stats(vault_owner VARCHAR(44))
RETURNS TABLE (
    total_deposits BIGINT,
    total_withdrawals BIGINT,
    transaction_count BIGINT,
    last_activity TIMESTAMP
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COALESCE(SUM(CASE WHEN tx_type = 'DEPOSIT' THEN amount ELSE 0 END), 0) as total_deposits,
        COALESCE(SUM(CASE WHEN tx_type = 'WITHDRAW' THEN amount ELSE 0 END), 0) as total_withdrawals,
        COUNT(*) as transaction_count,
        MAX(created_at) as last_activity
    FROM transactions
    WHERE user_pubkey = vault_owner;
END;
$$ LANGUAGE plpgsql;

-- Function: Check for balance discrepancies
CREATE OR REPLACE FUNCTION check_discrepancies()
RETURNS TABLE (
    vault_id UUID,
    user_pubkey VARCHAR(44),
    expected_balance BIGINT,
    actual_balance BIGINT,
    difference BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        va.id as vault_id,
        va.owner_pubkey as user_pubkey,
        va.total_balance as expected_balance,
        va.total_balance as actual_balance,
        0::BIGINT as difference
    FROM vault_accounts va
    WHERE va.status = 'ACTIVE';
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- SEED DATA FOR TESTING
-- ============================================

-- Insert initial system metrics
INSERT INTO system_metrics (metric_type, metric_value, metadata) VALUES
('TVL', 0, '{"currency": "USDT", "decimals": 6}'::jsonb),
('ACTIVE_VAULTS', 0, '{}'::jsonb),
('TOTAL_TRANSACTIONS', 0, '{}'::jsonb);

-- ============================================
-- PERMISSIONS (Optional - adjust as needed)
-- ============================================

-- Create read-only role for analytics
-- CREATE ROLE vault_reader;
-- GRANT SELECT ON ALL TABLES IN SCHEMA public TO vault_reader;
-- GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO vault_reader;

-- Create application role with full access
-- CREATE ROLE vault_app;
-- GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO vault_app;
-- GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO vault_app;

COMMENT ON TABLE vault_accounts IS 'Stores user vault account information and balances';
COMMENT ON TABLE transactions IS 'Complete transaction history for all vault operations';
COMMENT ON TABLE balance_snapshots IS 'Periodic snapshots of vault balances for historical analysis';
COMMENT ON TABLE reconciliation_logs IS 'Logs for on-chain vs off-chain balance reconciliation';
COMMENT ON TABLE audit_logs IS 'Complete audit trail of all system actions';
COMMENT ON TABLE system_metrics IS 'System-wide metrics and statistics';
COMMENT ON TABLE alerts IS 'System alerts for monitoring and security';
COMMENT ON TABLE cpi_operations IS 'Cross-program invocation operation logs';