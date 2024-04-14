use tonic::transport::Channel;

use anyhow::Result;

use coin::coin_client::CoinClient as CoinServiceClient;
use coin::{GetBalanceRequest, CreateInvoiceRequest, PayInvoiceRequest, GetInvoiceRequest};

pub mod coin {
    tonic::include_proto!("coin");
}

pub struct InvoicePayment {
    pub invoice: String,
    pub wallet: String,
    pub signature: String,
    pub pubkey: String,
}

pub struct InvoiceStatus {
    pub amount: f32,
    pub paid: bool,
}

pub struct CoinClient {
    client: CoinServiceClient<Channel>,
}

impl CoinClient {
    // Initialize a new CoinClient, connecting to the given coin service address
    pub async fn new(coin: String) -> Result<Self> {
        println!("gRPC Coin: Connecting to coin service at {}...", coin);
        let client = CoinServiceClient::connect(format!("http://{}", coin)).await?;

        Ok(CoinClient { client })
    }

    // Get the balance of a wallet
    pub async fn get_balance(&mut self, address: String) -> Result<f32> {
        println!("gRPC Coin: Getting balance for wallet address {}", address);

        let response = self
            .client
            .get_balance(GetBalanceRequest { wallet: address })
            .await?
            .into_inner();

        Ok(response.balance)
    }

    // Create an invoice for a given amount
    pub async fn create_invoice(&mut self, address: String, amount: f32) -> Result<String> {
        println!(
            "gRPC Coin: Creating invoice for wallet address {} with amount {}",
            address, amount
        );

        let response = self
            .client
            .create_invoice(CreateInvoiceRequest {
                wallet: address,
                amount,
            })
            .await?
            .into_inner();

        Ok(response.invoice)
    }

    // Pay an invoice
    pub async fn pay_invoice(&mut self, payment: InvoicePayment) -> Result<()> {
        println!(
            "gRPC Coin: Paying invoice {} from wallet {}",
            payment.invoice, payment.wallet
        );

        self.client
            .pay_invoice(PayInvoiceRequest {
                invoice: payment.invoice,
                wallet: payment.wallet,
                signature: payment.signature,
                pubkey: payment.pubkey,
            })
            .await?;

        Ok(())
    }

    // Check the status of an invoice
    pub async fn check_invoice(&mut self, invoice: String) -> Result<InvoiceStatus> {
        println!("gRPC Coin: Checking if invoice {} has been paid", invoice);

        let response = self
            .client
            .get_invoice(GetInvoiceRequest { invoice })
            .await?
            .into_inner();

        Ok(InvoiceStatus {
            amount: response.amount,
            paid: response.paid,
        })
    }
}
