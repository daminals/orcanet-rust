mod db;
pub mod files;
mod http;

use crate::market::Market;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};

pub async fn start_server(
    files: HashMap<String, PathBuf>,
    prices: HashMap<String, i64>,
    port: String,
) -> tokio::task::JoinHandle<()> {
    // Launch the HTTP server in the background
    let http_file_map = Arc::new(files::FileMap::new(files, prices));
    tokio::spawn(async move {
        if let Err(e) = http::run(http_file_map, port).await {
            eprintln!("HTTP server error: {}", e);
        }
    })
}

pub async fn stop_server(join_handle: tokio::task::JoinHandle<()>) -> Result<()> {
    // Stop the HTTP server
    join_handle.abort();
    Ok(())
}

pub async fn register_files(
    prices: HashMap<String, i64>,
    chunk_metadatas: HashMap<String, Vec<(String, u64)>>,
    client: &mut Market,
    port: String,
    ip: Option<String>,
) -> Result<()> {
    // get port from string
    let port = match port.parse::<i32>() {
        Ok(port) => port,
        Err(_) => {
            eprintln!("Invalid port number");
            return Ok(());
        }
    };

    // Get the public IP address
    let ip = match ip {
        Some(ip) => ip,
        // Use the AWS checkip service to get the public IP address
        None => match reqwest::get("http://checkip.amazonaws.com").await {
            Ok(resp) => match resp.text().await {
                Ok(text) => text.trim().to_string(),
                Err(e) => {
                    return Err(anyhow!("Failed to get public IP: {}", e));
                }
            },
            Err(e) => {
                return Err(anyhow!("Failed to get public IP: {}", e));
            }
        },
    };
    println!("Producer: IP address is {}", ip);

    // Generate a random Producer ID
    let producer_id = uuid::Uuid::new_v4().to_string();

    for (hash, price) in prices {
        println!(
            "Producer: Registering file with hash {} and price {}",
            hash, price
        );
        let chunk_metadata = match chunk_metadatas.get(&hash) {
            Some(chunk_metadata) => chunk_metadata.clone(),
            None => {
                eprintln!("Producer: No chunk metadata provided for file with hash {}", hash);
                continue;
            }
        };

        client
            .register_file(
                producer_id.clone(),
                "producer".to_string(),
                ip.clone(),
                port,
                price,
                hash,
                chunk_metadata
            )
            .await?;
    }

    Ok(())
}