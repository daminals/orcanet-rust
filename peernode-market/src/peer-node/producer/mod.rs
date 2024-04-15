mod db;
pub mod files;
mod http;

use std::sync::Arc;

use crate::market::market::Market;

use anyhow::{anyhow, Result};
use libp2p::Multiaddr;

pub async fn run(
    bootstrap_peers: &[Multiaddr],
    private_key: Option<String>,
    listen_address: Option<Multiaddr>,
    ip: Option<String>,
    port: Option<u16>,
) -> Result<()> {
    let mut client = Market::new(bootstrap_peers, private_key, listen_address).await?;

    // Load the files
    let file_map = Arc::new(files::FileMap::new());
    file_map.add_all("files/**/*").await?;

    // Get the port
    let port = port.unwrap_or(8080);

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
    client: &mut MarketClient,
    port: String,
    ip: Option<String>,
) -> Result<()> {
    // let mut client = MarketClient::new(market).await?;

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
        client
            .register_file(
                producer_id.clone(),
                "producer".to_string(),
                ip.clone(),
                port,
                price,
                hash,
            )
            .await?;
    }

    Ok(())
}
