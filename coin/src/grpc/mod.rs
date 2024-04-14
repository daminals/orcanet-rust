use std::sync::Arc;

use tonic::{Request, Response, Status};

use coin::coin_server::{Coin, CoinServer};
use coin::{
    CreateInvoiceReply, CreateInvoiceRequest, GetInvoiceReply, GetInvoiceRequest, PayInvoiceReply,
    PayInvoiceRequest, GetBalanceReply, GetBalanceRequest,
};
use anyhow::Result;

use crate::{crypto, db};

pub mod coin {
    tonic::include_proto!("coin");
}

pub struct CoinService {
    db: Arc<db::Database>,
}

#[tonic::async_trait]
impl Coin for CoinService {
    async fn create_invoice(
        &self,
        request: Request<CreateInvoiceRequest>,
    ) -> Result<Response<CreateInvoiceReply>, Status> {
        // Create a new invoice
        let request = request.into_inner();
        let id = self
            .db
            .create_invoice(request.amount, request.wallet)
            .await;
        let reply = CreateInvoiceReply { 
            invoice: id 
        };

        Ok(Response::new(reply))
    }

    // Get the status of an invoice
    async fn get_invoice(
        &self,
        request: Request<GetInvoiceRequest>,
    ) -> Result<Response<GetInvoiceReply>, Status> {
        let request = request.into_inner();
        let invoice = self.db.get_invoice(&request.invoice).await;
        let invoice = match invoice {
            Some(invoice) => invoice,
            None => return Err(Status::not_found("Invoice not found")),
        };

        let reply = GetInvoiceReply {
            amount: invoice.amount,
            paid: invoice.paid,
        };

        Ok(Response::new(reply))
    }

    async fn pay_invoice(
        &self,
        request: Request<PayInvoiceRequest>,
    ) -> Result<Response<PayInvoiceReply>, Status> {
        // Validate the payment
        let request = request.into_inner();
        let valid = crypto::validate_payment(
            &request.invoice,
            &request.wallet,
            &request.signature,
            &request.pubkey,
        );
        if !valid {
            return Err(Status::invalid_argument("Invalid payment"));
        }

        // Update the database
        if let Err(e) = self.db.pay_invoice(&request.invoice, &request.wallet).await {
            return Err(Status::invalid_argument(e.to_string()));
        }

        // Reply that the invoice was paid
        let reply = PayInvoiceReply { paid: true };
        Ok(Response::new(reply))
    }

    async fn get_balance(
        &self,
        request: Request<GetBalanceRequest>,
    ) -> Result<Response<GetBalanceReply>, Status> {
        // Get the balance
        let request = request.into_inner();
        let balance = self.db.get_balance(&request.wallet).await;
        let reply = GetBalanceReply { balance };

        Ok(Response::new(reply))
    }
}

impl CoinService {
    pub fn new(db: Arc<db::Database>) -> Self {
        Self { db }
    }
}

pub async fn serve(addr: String, db: Arc<db::Database>) -> Result<()> {
    let addr = addr.parse()?;

    let service = CoinService::new(db);

    tonic::transport::Server::builder()
        .add_service(CoinServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
