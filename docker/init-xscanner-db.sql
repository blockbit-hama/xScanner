-- xScanner Database Initialization Script
-- This script creates the necessary tables for xScanner

-- Create blockchain_state table
CREATE TABLE IF NOT EXISTS blockchain_state (
    chain_name VARCHAR(50) PRIMARY KEY,
    last_processed_block BIGINT NOT NULL
);

-- Create deposit_events table
CREATE TABLE IF NOT EXISTS deposit_events (
    id SERIAL PRIMARY KEY,
    address VARCHAR(255) NOT NULL,
    wallet_id VARCHAR(255) NOT NULL,
    account_id VARCHAR(255),
    chain_name VARCHAR(50) NOT NULL,
    tx_hash VARCHAR(255) NOT NULL,
    block_number BIGINT NOT NULL,
    amount VARCHAR(255) NOT NULL,
    amount_decimal NUMERIC(36, 18),
    confirmed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(chain_name, tx_hash)
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_de_address ON deposit_events (address);
CREATE INDEX IF NOT EXISTS idx_de_block_number ON deposit_events (block_number);
CREATE INDEX IF NOT EXISTS idx_de_confirmed ON deposit_events (confirmed) WHERE confirmed = FALSE;
CREATE INDEX IF NOT EXISTS idx_de_wallet_id ON deposit_events (wallet_id);
CREATE INDEX IF NOT EXISTS idx_de_account_id ON deposit_events (account_id) WHERE account_id IS NOT NULL;

-- Grant privileges
GRANT ALL PRIVILEGES ON TABLE blockchain_state TO user;
GRANT ALL PRIVILEGES ON TABLE deposit_events TO user;
GRANT ALL PRIVILEGES ON SEQUENCE deposit_events_id_seq TO user;
