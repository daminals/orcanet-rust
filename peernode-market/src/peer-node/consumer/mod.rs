pub mod encode;
pub mod http;

use crate::market::market::Market;

use anyhow::Result;
use libp2p::Multiaddr;

pub async fn run(
    bootstrap_peers: &[Multiaddr],
    private_key: Option<String>,
    listen_address: Option<Multiaddr>,
    file_hash: String,
) -> Result<()> {
    let client = Market::new(bootstrap_peers, private_key, listen_address).await?;

    // Check the producers for the file
    println!("Consumer: Checking producers for file hash {}", file_hash);
    let producers = client.check_holders(file_hash.clone()).await?;

    // For now, use the first producer
    // TODO: Allow user to choose a producer, give them a list of options with IP and port
    let producer = producers
        .holders
        .get(0)
        .ok_or(anyhow::anyhow!("No producers found"))?;
    println!(
        "Consumer: Found producer at {}:{}",
        producer.ip, producer.port
    );

    let mut chunk = 0;
    let mut token = String::from("token");
    // TODO: allow looping through chunks, but client should be allowed to cancel at any time
    // when the client cancels, the chunk num they stopped at should be returned to them so they
    // can query another producer for the next chunk
    loop {
        match http::get_file_chunk(producer.clone(), file_hash.clone(), token, chunk).await {
            Ok(response) => {
                match response {
                    http::GetFileResponse::Token(new_token) => {
                        token = new_token;
                    }
                    http::GetFileResponse::Done => {
                        println!("Consumer: File downloaded successfully");
                        break;
                    }
                }
                chunk += 1;
            }
            Err(e) => {
                eprintln!("Failed to download chunk {}: {}", chunk, e);
                break;
            }
        }
    }
    Ok(())
}

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
