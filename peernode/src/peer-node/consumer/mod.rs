pub mod encode;
pub mod http;

use crate::{globals::CHUNK_SIZE, grpc::{orcanet::User, MarketClient}, wallet::AsyncWallet};
use anyhow::Result;

use self::http::{GetFileResponse, GetInvoiceResponse};

pub async fn list_producers(file_hash: String, market: String) -> Result<()> {
    let mut client = MarketClient::new(market).await?;
    let producers = client.check_holders(file_hash).await?;
    for producer in producers.holders {
        // serialize the producer struct to a string
        let encoded_producer = encode::encode_user(&producer);
        println!(
            "Producer:\n  id: {}\n  Price: {}",
            encoded_producer, producer.price
        );
    }
    Ok(())
}

pub async fn get_invoice(producer: String, file_hash: String) -> Result<GetInvoiceResponse> {
    let producer_user = match encode::decode_user(producer.clone()) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Failed to decode producer: {}", e);
            return Err(anyhow::anyhow!("Failed to decode producer"));
        }
    };
    return http::get_invoice(producer_user, file_hash).await;
}

pub async fn get_file(
    producer: String,
    file_hash: String,
    token: String,
    chunk: u64,
    continue_download: bool,
) -> Result<()> {
    let producer_user = match encode::decode_user(producer.clone()) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Failed to decode producer: {}", e);
            return Err(anyhow::anyhow!("Failed to decode producer"));
        }
    };
    let mut chunk_num = chunk;
    loop {
        match get_file_chunk(
            producer_user.clone(),
            file_hash.clone(),
            token.clone(),
            chunk_num,
        )
        .await
        {
            Ok(response) => {
                match response {
                    GetFileResponse::InProgress => {
                        chunk_num += 1;
                    }
                    GetFileResponse::Done => {
                        println!("Consumer: File downloaded successfully");
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to download chunk {}: {}", chunk_num, e);
                return Err(anyhow::anyhow!("Failed to download chunk"));
            }
        }
        if continue_download == false {
            return Ok(());
        }
    }
}

pub async fn get_file_chunk(
    producer: User,
    file_hash: String,
    token: String,
    chunk: u64,
) -> Result<GetFileResponse> {
    return http::get_file_chunk(producer, file_hash.clone(), token, chunk).await;
}

pub async fn get_file_auto(
    producer: String,
    file_hash: String,
    wallet: AsyncWallet,
) -> Result<String> {
    let producer_user = match encode::decode_user(producer.clone()) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Failed to decode producer: {}", e);
            return Err(anyhow::anyhow!("Failed to decode producer"));
        }
    };

    // Get the invoice
    let invoice = get_invoice(producer.clone(), file_hash.clone()).await?;
    let token = invoice.token.clone();
    let invoice = invoice.invoice.clone();

    // Find out how much we owe
    let mut wallet = wallet.write().await;
    let total_owed = wallet.check_invoice(invoice.clone()).await?.amount;

    // Calculate price per chunk
    let price_per_chunk = CHUNK_SIZE as f32 * producer_user.price as f32 / 1024.0 / 1024.0;

    let mut total_paid = 0.0;
    let mut chunk = 0;
    loop {
        // Pay for the chunk
        let mut amount = price_per_chunk;
        if total_paid + price_per_chunk > total_owed {
            // Pay the remaining amount
            amount = total_owed - total_paid;
        }
        if amount >= 0.0 {
            wallet.pay_invoice(invoice.clone(), Some(amount)).await?;
        }

        // Get the next chunk
        match get_file_chunk(
            producer_user.clone(),
            file_hash.clone(),
            token.clone(),
            chunk,
        )
        .await
        {
            Ok(response) => {
                match response {
                    GetFileResponse::InProgress => {
                        // Update the amount paid
                        total_paid += price_per_chunk;
                        chunk += 1;
                    }
                    GetFileResponse::Done => {
                        println!("Consumer: File downloaded successfully");
                        return Ok(token);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to download chunk {}: {}", chunk, e);
                return Err(anyhow::anyhow!("Failed to download chunk"));
            }
        }
    }
}

// pub async fn upload_file(file_path: String, market: String) -> Result<()> {
//     let mut client = MarketClient::new(market).await?;
//     //let file_hash = client.upload_file(file_path).await?;
//     println!("File uploaded successfully, hash: {}", file_hash);
//     Ok(())
// }
