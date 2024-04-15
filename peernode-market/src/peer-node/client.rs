mod consumer;
mod market;
mod producer;

use anyhow::{anyhow, Result};
use clap::Parser;
use libp2p::Multiaddr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Peer node client
struct Args {
    /// Bootstrap nodes to connect to
    #[arg(short, long, num_args = 0..)]
    bootstrap_peers: Vec<Multiaddr>,

    /// Private key file, should be in rsa pkcs8 format
    #[arg(short = 'k', long)]
    private_key: Option<String>,

    /// Multiaddr for listen address
    /// Only used when running as a producer
    #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/6881")]
    listen_address: Option<Multiaddr>,

    /// Whether to run as a producer
    #[arg(short, long, default_value = "false")]
    producer: bool,

    /// File hash
    /// Only used when running as a consumer
    #[arg(short, long)]
    file_hash: Option<String>,

    /// IP address which should be provided to the market service
    /// If not provided, the producer will find its own public IP address
    #[arg(long, requires("producer"))]
    ip: Option<String>,

    /// Port the producer should listen on
    /// If not provided, the producer will listen on 8080
    #[arg(long, requires("producer"))]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();

    match args.producer {
        true => {
            producer::run(
                &args.bootstrap_peers,
                args.private_key,
                args.listen_address,
                args.ip,
                args.port,
            )
            .await?
        }
        false => match args.file_hash {
            Some(file_hash) => {
                consumer::run(&args.bootstrap_peers, args.private_key, None, file_hash).await?
            }
            None => return Err(anyhow!("No file hash provided")),
        },
    }

    Ok(())
}
