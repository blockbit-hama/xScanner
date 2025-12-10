use crate::respository::{RepositoryWrapper, Repository};
use crate::notification::sqs_client::SqsNotifier;
use crate::config::ChainConfig;
use log::{info, error, warn};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::time::{interval, Duration};

/// Pending deposit information from database
#[derive(Debug, Clone)]
pub struct PendingDeposit {
    pub address: String,
    pub wallet_id: String,
    pub account_id: Option<String>,
    pub chain_name: String,
    pub tx_hash: String,
    pub block_number: u64,
    pub amount: String,
    pub amount_decimal: Option<rust_decimal::Decimal>,
}

/// Configuration for confirmation checker
pub struct ConfirmationCheckerConfig {
    pub enabled: bool,
    pub check_interval_secs: u64,
}

impl Default for ConfirmationCheckerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: 30,
        }
    }
}

/// Run confirmation checker - periodically checks pending deposits
///
/// This service periodically queries the database for unconfirmed deposits
/// and checks if they have reached the required number of confirmations.
pub async fn run_confirmation_checker(
    repository: Arc<RepositoryWrapper>,
    chain_configs: HashMap<String, ChainConfig>,
    sqs_notifier: Option<Arc<SqsNotifier>>,
    config: ConfirmationCheckerConfig,
) {
    if !config.enabled {
        info!("[ConfirmationChecker] Disabled by configuration, skipping...");
        return;
    }

    info!("[ConfirmationChecker] Starting with check_interval: {}s", config.check_interval_secs);

    let mut check_interval = interval(Duration::from_secs(config.check_interval_secs));

    loop {
        check_interval.tick().await;

        if let Err(e) = check_pending_deposits(
            &repository,
            &chain_configs,
            sqs_notifier.as_deref(),
        ).await {
            error!("[ConfirmationChecker] Error checking pending deposits: {}", e);
        }
    }
}

/// Check all pending deposits and send confirmations if ready
async fn check_pending_deposits(
    repository: &Arc<RepositoryWrapper>,
    chain_configs: &HashMap<String, ChainConfig>,
    sqs_notifier: Option<&SqsNotifier>,
) -> Result<(), String> {
    // Get all pending (unconfirmed) deposits from database
    let pending_deposits = repository
        .get_pending_deposits()
        .await
        .map_err(|e| format!("Failed to get pending deposits: {}", e))?;

    if pending_deposits.is_empty() {
        info!("[ConfirmationChecker] No pending deposits to check");
        return Ok(());
    }

    info!("[ConfirmationChecker] Checking {} pending deposits", pending_deposits.len());

    for deposit in pending_deposits {
        // Get chain config for required confirmations
        let required_confirmations = chain_configs.get(&deposit.chain_name.to_uppercase())
            .or_else(|| chain_configs.get(&deposit.chain_name.to_lowercase()))
            .map(|c| c.required_confirmations)
            .unwrap_or(12); // Default to 12 if not found

        // Get current block number for this chain
        let current_block = match repository.get_last_processed_block(&deposit.chain_name).await {
            Ok(block) => block,
            Err(e) => {
                error!("[ConfirmationChecker] Failed to get last processed block for {}: {}", deposit.chain_name, e);
                continue;
            }
        };

        // Calculate confirmations
        let confirmations = current_block.saturating_sub(deposit.block_number) + 1;

        info!(
            "[ConfirmationChecker] Deposit {} on {} - confirmations: {}/{} (block: {}, current: {})",
            deposit.tx_hash, deposit.chain_name, confirmations, required_confirmations,
            deposit.block_number, current_block
        );

        // Check if reached required confirmations
        if confirmations >= required_confirmations {
            // Check if already confirmed (double-check to prevent duplicates)
            let is_confirmed = repository
                .is_deposit_confirmed(&deposit.tx_hash)
                .await
                .map_err(|e| format!("Failed to check confirmation status: {}", e))?;

            if is_confirmed {
                warn!("[ConfirmationChecker] Deposit {} already confirmed, skipping", deposit.tx_hash);
                continue;
            }

            info!(
                "[ConfirmationChecker] ✅ Deposit {} reached {} confirmations, sending DEPOSIT_CONFIRMED",
                deposit.tx_hash, confirmations
            );

            // Update database
            repository
                .update_deposit_confirmed(&deposit.tx_hash)
                .await
                .map_err(|e| format!("Failed to update deposit confirmation: {}", e))?;

            // Send SQS notification
            if let Some(notifier) = sqs_notifier {
                if let Err(e) = notifier.send_deposit_confirmed(
                    deposit.address.clone(),
                    deposit.wallet_id.clone(),
                    deposit.account_id.clone(),
                    deposit.chain_name.to_uppercase(),
                    deposit.tx_hash.clone(),
                    deposit.amount.clone(),
                    deposit.block_number,
                    confirmations,
                ).await {
                    error!("[ConfirmationChecker] Failed to send SQS notification: {}", e);
                } else {
                    info!("[ConfirmationChecker] ✅ SQS DEPOSIT_CONFIRMED sent for {}", deposit.tx_hash);
                }
            }
        } else {
            info!(
                "[ConfirmationChecker] Deposit {} needs {} more confirmations",
                deposit.tx_hash, required_confirmations - confirmations
            );
        }
    }

    Ok(())
}
