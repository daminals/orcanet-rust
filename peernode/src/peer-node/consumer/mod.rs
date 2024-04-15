pub mod encode;
pub mod http;

use crate::grpc::{orcanet::User, MarketClient};
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

// pub async fn upload_file(file_path: String, market: String) -> Result<()> {
//     let mut client = MarketClient::new(market).await?;
//     //let file_hash = client.upload_file(file_path).await?;
//     println!("File uploaded successfully, hash: {}", file_hash);
//     Ok(())
// }
