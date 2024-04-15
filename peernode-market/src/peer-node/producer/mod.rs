mod db;
mod files;
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
    let http_file_map = file_map.clone();
    tokio::spawn(async move {
        if let Err(e) = http::run(http_file_map, port).await {
            eprintln!("HTTP server error: {}", e);
        }
    });

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

    // Generate a random producer ID
    let producer_id = uuid::Uuid::new_v4().to_string();

    // Register the files with the market service
    let hash = file_map.get_hashes().await;
    for hash in hash {
        // TODO: Find a way to get the public IP address
        println!("Producer: Registering file with hash {}", hash);
        client
            .register_file(
                producer_id.clone(),
                "producer".to_string(),
                ip.clone(),
                port.into(),
                100,
                hash,
            )
            .await?;
    }

    // Never return
    tokio::signal::ctrl_c().await?;

    Ok(())
}
