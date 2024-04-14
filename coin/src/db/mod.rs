use std::collections::HashMap;

use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::{anyhow, Result};

#[derive(Clone, Debug)]
pub struct Invoice {
    pub id: String,
    pub amount: f32,
    pub amount_paid: f32,
    pub wallet: String,
    pub paid: bool,
}

pub struct Database {
    pub balances: RwLock<HashMap<String, f32>>,
    pub invoices: RwLock<HashMap<String, Invoice>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            balances: RwLock::new(HashMap::new()),
            invoices: RwLock::new(HashMap::new()),
        }
    }

    // Create a new invoice with a random ID
    pub async fn create_invoice(&self, amount: f32, wallet: String) -> String {
        let id = Uuid::new_v4().to_string();
        let invoice = Invoice {
            id: id.clone(),
            amount,
            amount_paid: 0.0,
            wallet,
            paid: false,
        };

        let mut invoices = self.invoices.write().await;
        invoices.insert(id.clone(), invoice);

        id
    }

    // Pay an invoice
    pub async fn pay_invoice(&self, id: &str, wallet: &str, amount: Option<f32>) -> Result<()> {
        // Get the invoice
        let mut invoices = self.invoices.write().await;
        let invoice = invoices.get_mut(id).ok_or(anyhow!("Invoice not found"))?;

        // Check if the invoice has already been paid
        if invoice.paid {
            return Err(anyhow!("Invoice already paid"));
        }

        // Check for overpayment
        if let Some(amount) = amount {
            if amount + invoice.amount_paid > invoice.amount {
                return Err(anyhow!("Overpayment"));
            }
        }

        // Calculate the amount to pay (pay the full amount if not specified)
        let amount = amount.unwrap_or(invoice.amount - invoice.amount_paid);

        // Check for sufficient funds
        let mut balances = self.balances.write().await;
        let balance = balances.entry(wallet.to_string()).or_insert(0.0);
        if *balance < amount {
            return Err(anyhow!("Insufficient funds"));
        }

        // Deduct the amount from the balance
        *balance -= amount;

        // Add the amount to the invoice creator's balance
        let creator = invoice.wallet.clone();
        let creator_balance = balances.entry(creator).or_insert(0.0);
        *creator_balance += amount;

        // Update the invoice
        invoice.amount_paid += amount;
        if invoice.amount_paid >= invoice.amount {
            invoice.paid = true;
        }

        Ok(())
    }

    // Get an invoice
    pub async fn get_invoice(&self, id: &str) -> Option<Invoice> {
        let invoices = self.invoices.read().await;
        invoices.get(id).cloned()
    }

    // Get the balance of a wallet
    pub async fn get_balance(&self, wallet: &str) -> f32 {
        let mut balances = self.balances.write().await;
        balances.entry(wallet.to_string()).or_insert(0.0).clone()
    }

    // Add funds to a wallet
    pub async fn add_funds(&self, wallet: &str, amount: f32) {
        let mut balances = self.balances.write().await;
        let balance = balances.entry(wallet.to_string()).or_insert(0.0);
        *balance += amount;
    }
}