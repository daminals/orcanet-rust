use clap::Parser;
use orcanet_market::Multiaddr;

use crate::Port;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value = "50051")]
    pub market_port: Port,
    #[arg(short, long, default_value = "16899")]
    pub peer_port: Port,
    #[arg(short, long, value_parser, num_args = 0.., value_delimiter = ',')]
    pub boot_nodes: Option<Vec<Multiaddr>>,
    #[arg(long)]
    pub public_address: Option<Multiaddr>,
}
