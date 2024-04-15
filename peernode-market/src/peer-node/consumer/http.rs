use anyhow::{anyhow, Result};

use crate::market::User;

use std::time::Instant;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub enum GetFileResponse {
    Token(String),
    Done,
}

pub async fn get_file_chunk(
    producer: User,
    file_hash: String,
    token: String,
    chunk: u64,
) -> Result<GetFileResponse> {
    let start = Instant::now();
    // Get the link to the file
    let link = format!(
        "http://{}:{}/file/{}?chunk={}",
        producer.ip, producer.port, file_hash, chunk
    );
    println!("HTTP: Fetching file chunk from {}", link);

    // Fetch the file from the producer
    let client = reqwest::Client::new();
    let res = client
        .get(&link)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    // Check if the request was successful
    if !res.status().is_success() {
        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(GetFileResponse::Done);
        }
        return Err(anyhow!("Request failed with status code: {}", res.status()));
    }

    // Get auth token header from response
    let headers = res.headers().clone();
    let auth_token = headers
        .get("X-Access-Token")
        .ok_or(anyhow!("No Authorization header"))?
        .to_str()?;

    // Get the file name from the Content-Disposition header
    let content_disposition = headers
        .get("Content-Disposition")
        .ok_or(anyhow!("No Content-Disposition header"))?
        .to_str()?;
    let file_name = match content_disposition.split("filename=").last() {
        Some(name) => name,
        None => return Err(anyhow!("No filename in Content-Disposition header")),
    };
    let file_name = file_name.trim_matches(|c| c == '"'); // Remove quotes

    // Save the file to disk
    let file = res.bytes().await?;
    let file_path = format!("download/{}", file_name);
    let mut download = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .await?;

    download.write_all(&file).await?;
    let duration = start.elapsed();
    println!(
        "HTTP: Chunk [{}] saved to {} [{} ms]",
        chunk,
        file_name,
        duration.as_millis()
    );
    Ok(GetFileResponse::Token(auth_token.to_string()))
}
