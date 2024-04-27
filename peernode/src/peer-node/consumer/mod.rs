pub mod encode;
pub mod http;

use anyhow::Result;
use orcanet_market::SupplierInfo;
use std::fmt::Write;

use crate::peer::MarketClient;

use self::http::GetFileResponse;

// list every producer who holds the file hash I want
pub async fn list_producers(file_hash: String, client: &mut MarketClient) -> Result<String> {
    let producers = client.check_holders(file_hash.clone()).await?;
    let mut producer_list = String::new();
    for producer in producers {
        // serialize the producer struct to a string
        let encoded_producer = encode::encode_user(&producer);
        if let Err(e) = writeln!(
            &mut producer_list,
            "Producer:\n  id: {}\n  Price: {}\n",
            encoded_producer, producer.price
        ) {
            eprintln!("Failed to write producer: {}", e);
            return Err(anyhow::anyhow!("Failed to write producer"));
        }
        println!(
            "Producer:\n  id: {}\n  Price: {}",
            encoded_producer, producer.price
        );
    }
    Ok(producer_list)
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
    let mut return_token = String::from(token);
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
        if continue_download == false {
            return Ok(return_token);
        }
    }
}

// get individual chunk of file from producer by hash
pub async fn get_file_chunk(
    producer: SupplierInfo,
    file_hash: String,
    token: String,
    chunk: u64,
) -> Result<GetFileResponse> {
    return http::get_file_chunk(producer, file_hash.clone(), token, chunk).await;
}

// TODO: implement upload_file
// pub async fn upload_file(file_path: String, market: String) -> Result<()> {
//     let mut client = MarketClient::new(market).await?;
//     //let file_hash = client.upload_file(file_path).await?;
//     println!("File uploaded successfully, hash: {}", file_hash);
//     Ok(())
// }
