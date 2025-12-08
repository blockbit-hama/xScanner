use aws_sdk_sqs::Client as SqsClient;
use serde::{Serialize, Deserialize};
use log::{info, error};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum DepositEvent {
    DepositDetected {
        customer_id: String,
        address: String,
        chain: String,
        tx_hash: String,
        amount: String,
        block_number: u64,
        confirmations: u64,
    },
    DepositConfirmed {
        customer_id: String,
        address: String,
        chain: String,
        tx_hash: String,
        amount: String,
        block_number: u64,
        confirmations: u64,
    },
}

pub struct SqsNotifier {
    client: SqsClient,
    queue_url: String,
}

impl SqsNotifier {
    pub async fn new(queue_url: String, region: String) -> Result<Self, String> {
        let config = aws_config::from_env()
            .region(aws_config::Region::new(region))
            .load()
            .await;

        let client = SqsClient::new(&config);

        Ok(Self { client, queue_url })
    }

    pub async fn send_deposit_event(&self, event: DepositEvent) -> Result<(), String> {
        let message_body = serde_json::to_string(&event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;

        info!("Sending SQS message: {}", message_body);

        self.client
            .send_message()
            .queue_url(&self.queue_url)
            .message_body(message_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send SQS message: {}", e))?;

        info!("✅ SQS message sent successfully");
        Ok(())
    }

    pub async fn send_deposit_detected(
        &self,
        customer_id: String,
        address: String,
        chain: String,
        tx_hash: String,
        amount: String,
        block_number: u64,
    ) -> Result<(), String> {
        let event = DepositEvent::DepositDetected {
            customer_id,
            address,
            chain,
            tx_hash,
            amount,
            block_number,
            confirmations: 1,
        };

        self.send_deposit_event(event).await
    }

    pub async fn send_deposit_confirmed(
        &self,
        customer_id: String,
        address: String,
        chain: String,
        tx_hash: String,
        amount: String,
        block_number: u64,
        confirmations: u64,
    ) -> Result<(), String> {
        let event = DepositEvent::DepositConfirmed {
            customer_id,
            address,
            chain,
            tx_hash,
            amount,
            block_number,
            confirmations,
        };

        self.send_deposit_event(event).await
    }
}
