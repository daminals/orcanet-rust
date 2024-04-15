use rand::Rng;
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::wallet::AsyncWallet;

use super::files::{AsyncFileMap, FileAccessType};
use anyhow::{anyhow, Result};

pub struct AppState {
    // Map of File Hash + Token -> Consumer
    consumers: RwLock<HashMap<String, Consumer>>,

    // Wallet
    wallet: AsyncWallet,

    // File map
    files: AsyncFileMap,
}

#[derive(Clone)]
pub struct Consumer {
    pub token: String,
    pub invoice: String,
    pub price_per_mb: u64,
    pub data_sent_mb: f32,
}

impl AppState {
    pub fn new(wallet: AsyncWallet, files: AsyncFileMap) -> Self {
        AppState {
            consumers: RwLock::new(HashMap::new()),
            wallet,
            files,
        }
    }

    // Get a file access object
    pub async fn get_file_access(&self, file_hash: &str) -> Result<FileAccessType> {
        // Locate the file requested
        let path = match self.files.get_file_path(file_hash).await {
            Some(path) => path,
            None => return Err(anyhow!("File not found")),
        };

        // Open the file
        let file = match FileAccessType::new(&path.to_string_lossy().to_string()) {
            Ok(file) => file,
            Err(_) => return Err(anyhow!("Failed to open file")),
        };

        Ok(file)
    }

    // Generate a token + invoice for a specific file
    pub async fn generate_invoice(&self, file_hash: &str) -> Result<Consumer> {
        // Locate the file requested
        let file = self.get_file_access(file_hash).await?;

        // Calculate the price per MB
        let price_per_mb = match self.files.get_price(file_hash).await {
            Some(price) => price,
            None => return Err(anyhow!("Price not found")),
        };
        // TODO: Don't generate an invoice if the price is 0

        // Calculate the total price
        let size = file.get_size().await?;
        let size_mb = size as f32 / 1024.0 / 1024.0;
        let total_price = size_mb * price_per_mb as f32;

        // Generate a token
        let token = generate_access_token();

        // Generate an invoice
        let invoice = self
            .wallet
            .write()
            .await
            .create_invoice(total_price)
            .await?;

        // Save the token and invoice
        let mut consumers = self.consumers.write().await;
        let consumer = Consumer {
            token: token.clone(),
            invoice,
            price_per_mb: price_per_mb as u64,
            data_sent_mb: 0.0,
        };
        let hash = format!("{}{}", file_hash, token);
        consumers.insert(hash.clone(), consumer.clone());

        Ok(consumer)
    }

    // Verify payment for a consumer to a chunk of a file
    pub async fn verify_payment(&self, file_hash: &str, token: &str, chunk: u64) -> Result<()> {
        // Get the consumer
        let hash = format!("{}{}", file_hash, token);
        let mut consumers = self.consumers.write().await;
        let consumer = match consumers.get_mut(&hash) {
            Some(consumer) => consumer,
            None => return Err(anyhow!("Consumer not found")),
        };

        // Calculate the value of what has been sent
        let value_sent = consumer.price_per_mb as f32 * consumer.data_sent_mb as f32;

        // Check how much they've paid us
        let mut wallet = self.wallet.write().await;
        let invoice = wallet.check_invoice(consumer.invoice.clone()).await?;
        let value_paid = invoice.amount_paid;

        // Check if they've paid enough
        let file = self.get_file_access(file_hash).await?;
        let chunk_size_mb = file.get_chunk_size(chunk).await? as f32 / 1024.0 / 1024.0;
        let chunk_value = consumer.price_per_mb as f32 * chunk_size_mb;
        if value_paid < value_sent + chunk_value {
            println!(
                "Consumer: Insufficient payment. Paid: {}, Required: {}",
                value_paid,
                value_sent + chunk_value as f32
            );
            return Err(anyhow!("Insufficient payment"));
        }

        // Update the amount of data sent
        consumer.data_sent_mb += file.get_chunk_size(chunk).await? as f32 / 1024.0 / 1024.0;

        Ok(())
    }
}

pub fn generate_access_token() -> String {
    // Generate a completely random access token
    let mut rng = rand::thread_rng();
    let token: String = (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();

    token
}
