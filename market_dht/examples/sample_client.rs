use std::{
    thread::{self, sleep},
    time::Duration,
};

use libp2p::Multiaddr;
use market_dht::{config::Config, multiaddr, net::spawn_bridge};
use tracing_log::LogTracer;

fn main() {
    LogTracer::init().unwrap();
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_thread_names(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let peer1 = spawn_bridge(
        Config::builder(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(5555u16))).build(),
        "peer1".to_owned(),
    )
    .unwrap();
    thread::sleep(Duration::from_secs(3));
    let peer1_id = peer1.id();
    let peer2 = spawn_bridge(
        Config::builder(multiaddr!(Ip4([127, 0, 0, 1]), Tcp(1234u16)))
            .with_boot_nodes(
                vec![("/ip4/127.0.0.1/tcp/5555".to_owned(), peer1_id.to_string())]
                    .try_into()
                    .unwrap(),
            )
            .build(),
        "peer2".to_owned(),
    )
    .unwrap();
    thread::sleep(Duration::from_secs(7777777));
}
