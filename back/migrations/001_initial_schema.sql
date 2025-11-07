-- migrations/001_initial_schema.sql
-- Initial database schema for Collateral Vault Management System

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
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES vault_accounts(id) ON DELETE CASCADE,
    user_pubkey VARCHAR(44) NOT NULL,
    tx_type VARCHAR(20) NOT NULL,
    amount BIGINT NOT NULL,
    signature VARCHAR(88) NOT NULL UNIQUE,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING',
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS balance_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES vault_accounts(id) ON DELETE CASCADE,
    user_pubkey VARCHAR(44) NOT NULL,
    total_balance BIGINT NOT NULL,
    locked_balance BIGINT NOT NULL,
    available_balance BIGINT NOT NULL,
    snapshot_type VARCHAR(10) NOT NULL,
    snapshot_time TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_pubkey VARCHAR(44) NOT NULL,
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(30) NOT NULL,
    details TEXT,
    ip_address VARCHAR(45),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS reconciliation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_id UUID NOT NULL REFERENCES vault_accounts(id) ON DELETE CASCADE,
    user_pubkey VARCHAR(44) NOT NULL,
    onchain_balance BIGINT NOT NULL,
    offchain_balance BIGINT NOT NULL,
    discrepancy BIGINT NOT NULL,
    status VARCHAR(20) NOT NULL,
    checked_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type VARCHAR(30) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    vault_id UUID REFERENCES vault_accounts(id) ON DELETE CASCADE,
    user_pubkey VARCHAR(44),
    message TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_vault_owner ON vault_accounts(owner_pubkey);
CREATE INDEX IF NOT EXISTS idx_tx_vault ON transactions(vault_id);
CREATE INDEX IF NOT EXISTS idx_tx_user ON transactions(user_pubkey);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_logs(user_pubkey);