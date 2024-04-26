mod db;
pub mod files;
mod http;
pub mod jobs;

use orcanet_market::Market;
use proto::market::FileInfo;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};

pub async fn start_server(
    files: HashMap<String, PathBuf>,
    prices: HashMap<String, i64>,
    file_names: HashMap<String, String>,
    port: String,
) -> tokio::task::JoinHandle<()> {
    // Launch the HTTP server in the background
    let http_file_map = Arc::new(files::FileMap::new(files, prices, file_names));
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
    file_info_map: HashMap<String, FileInfo>,
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
    //let ip = "0.0.0.0".to_string();
    println!("Producer: IP address is {}", ip);

    // Generate a random Producer ID
    let producer_id = uuid::Uuid::new_v4().to_string();

    for (hash, price) in prices {
        println!(
            "Producer: Registering file with hash {} and price {}",
            hash, price
        );
        let file_info = match file_info_map.get(&hash) {
            Some(file_info) => file_info.clone(),
            None => {
                eprintln!(
                    "Producer: No file_info provided for file with hash {}",
                    hash
                );
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
                file_info,
            )
            .await?;
    }

    Ok(())
}
