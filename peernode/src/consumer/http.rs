use anyhow::{anyhow, Result};
use proto::market::{FileInfoHash, User};

use std::path::PathBuf;
use std::time::Instant;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub enum GetFileResponse {
    Token(String),
    Done,
}

pub async fn get_file_chunk(
    producer: User,
    file_info_hash: FileInfoHash,
    token: String,
    chunk: u64,
) -> Result<GetFileResponse> {
    let start = Instant::now();
    // Get the link to the file
    let link = format!(
        "http://{}:{}/file/{file_info_hash}?chunk={chunk}",
        producer.ip, producer.port
    );
    println!("HTTP: Fetching file chunk from {link}");

    // Fetch the file from the producer
    let client = reqwest::Client::new();
    let res = client
        .get(&link)
        .header("Authorization", format!("Bearer {token}"))
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
    // for now, add customization option?
    if !PathBuf::from("download").exists() {
        return Err(anyhow!("download folder does not exist, cannot proceed"));
    }
    let file_path: PathBuf = ["download", file_name].iter().collect();
    let mut download = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .await?;

    download.write_all(&file).await?;
    let duration = start.elapsed().as_millis();
    println!("HTTP: Chunk [{chunk}] saved to {file_name} [{duration} ms]");
    Ok(GetFileResponse::Token(auth_token.to_string()))
}
