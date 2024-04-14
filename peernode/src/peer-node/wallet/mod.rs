pub mod client;

use std::sync::Arc;

use anyhow::{anyhow, Result};
use ring::{
    digest::{digest, SHA256},
    rand,
    signature::{self, KeyPair},
};
use tokio::sync::RwLock;

pub struct Wallet {
    pub address: String,
    pub pubkey: String,
    keypair: signature::Ed25519KeyPair,
    client: client::CoinClient,
}

pub type AsyncWallet = Arc<RwLock<Wallet>>;

impl Wallet {
    pub async fn new(server: String) -> Result<Wallet> {
        // Check if the wallet file exists
        if std::path::Path::new("wallet.pkcs8").exists() {
            return Wallet::from_file("wallet.pkcs8", server).await;
        }

        // Generate a new keypair
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = match signature::Ed25519KeyPair::generate_pkcs8(&rng) {
            Ok(pkcs8_bytes) => pkcs8_bytes,
            Err(_) => {
                return Err(anyhow!("Failed to generate keypair"));
            }
        };

        // Save the PKCS#8 bytes
        match std::fs::write("wallet.pkcs8", pkcs8_bytes.as_ref()) {
            Ok(_) => {}
            Err(_) => {
                panic!("Failed to write wallet.pkcs8");
            }
        }

        // Load the keypair
        return Wallet::from_pkcs8(pkcs8_bytes.as_ref(), server).await;
    }

    // Load a wallet from PKCS#8 bytes
    pub async fn from_pkcs8(pkcs8_bytes: &[u8], server: String) -> Result<Wallet> {
        // Load the keypair
        let keypair = match signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes) {
            Ok(keypair) => keypair,
            Err(_) => {
                return Err(anyhow!("Failed to load keypair"));
            }
        };

        // Get the public key
        let pubkey = keypair.public_key().as_ref();

        // Get the address (SHA256 hash of the public key)
        let sha256_digest = digest(&SHA256, pubkey);
        let address = hex::encode(sha256_digest.as_ref());

        // Launch the CoinClient
        let client = client::CoinClient::new(server).await?;

        Ok(Wallet {
            address,
            pubkey: hex::encode(pubkey),
            keypair,
            client,
        })
    }

    // Load a wallet from a PKCS#8 file
    pub async fn from_file(filename: &str, server: String) -> Result<Wallet> {
        let pkcs8_bytes = std::fs::read(filename)?;
        Wallet::from_pkcs8(&pkcs8_bytes, server).await
    }

    // Create an invoice for this wallet
    pub async fn create_invoice(&mut self, amount: f32) -> Result<String> {
        // Create the invoice
        let invoice = self
            .client
            .create_invoice(self.address.clone(), amount)
            .await?;
        Ok(invoice)
    }

    // Pay an invoice
    pub async fn pay_invoice(&mut self, invoice: String, amount: Option<f32>) -> Result<()> {
        // Sign the invoice + wallet address
        let combined = match amount {
            Some(amount) => format!("{}{}{}", invoice, self.address.clone(), amount),
            None => format!("{}{}", invoice, self.address.clone()),
        };
        let signature = self.sign(combined.as_bytes())?;

        // Pay the invoice
        self.client
            .pay_invoice(client::InvoicePayment {
                invoice,
                wallet: self.address.clone(),
                amount,
                signature: hex::encode(signature),
                pubkey: self.pubkey.clone(),
            })
            .await?;

        Ok(())
    }

    // Check the status of an invoice
    pub async fn check_invoice(&mut self, invoice: String) -> Result<client::InvoiceStatus> {
        // Get the status of the invoice
        let status = self.client.check_invoice(invoice).await?;
        Ok(status)
    }

    // Get the balance of this wallet
    pub async fn get_balance(&mut self) -> Result<f32> {
        // Get the balance
        let balance = self.client.get_balance(self.address.clone()).await?;
        Ok(balance)
    }

    // Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        let signature = self.keypair.sign(message);
        Ok(signature.as_ref().to_vec())
    }
}
