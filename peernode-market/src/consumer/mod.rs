pub mod encode;
pub mod http;

use crate::market::{Market, User};
use anyhow::Result;

use self::http::GetFileResponse;

pub async fn list_producers(file_hash: String, client: &mut Market) -> Result<()> {
    let producers = match client.check_holders(file_hash.clone()).await? {
        Some(producers) => producers,
        None => {
            println!("No producers for file {file_hash}");
            return Ok(());
        }
    };
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

// get file I want by hash from producer
pub async fn get_file(
    producer: String,
    file_hash: String,
    token: String,
    chunk: u64,
    continue_download: bool,
) -> Result<String> {
    let producer_user = match encode::decode_user(producer.clone()) {
        Ok(user) => user,
        Err(e) => {
            eprintln!("Failed to decode producer: {}", e);
            return Err(anyhow::anyhow!("Failed to decode producer"));
        }
    };
    let mut chunk_num = chunk;
    let mut return_token = token;
    loop {
        match get_file_chunk(
            producer_user.clone(),
            file_hash.clone(),
            return_token.clone(),
            chunk_num,
        )
        .await
        {
            Ok(response) => {
                match response {
                    GetFileResponse::Token(new_token) => {
                        return_token = new_token;
                    }
                    GetFileResponse::Done => {
                        println!("Consumer: File downloaded successfully");
                        return Ok(return_token);
                    }
                }
                chunk_num += 1;
            }
            Err(e) => {
                eprintln!("Failed to download chunk {}: {}", chunk_num, e);
                return Err(anyhow::anyhow!("Failed to download chunk"));
            }
        }
        if !continue_download {
            return Ok(return_token);
        }
    }
}

// get individual chunk of file from producer by hash
pub async fn get_file_chunk(
    producer: User,
    file_hash: String,
    token: String,
    chunk: u64,
) -> Result<GetFileResponse> {
    http::get_file_chunk(producer, file_hash.clone(), token, chunk).await
}

// TODO: implement upload_file
// pub async fn upload_file(file_path: String, market: String) -> Result<()> {
//     let mut client = MarketClient::new(market).await?;
//     //let file_hash = client.upload_file(file_path).await?;
//     println!("File uploaded successfully, hash: {}", file_hash);
//     Ok(())
// }
