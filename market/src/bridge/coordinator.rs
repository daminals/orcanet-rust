use std::{net::Ipv4Addr, time::Duration};

use anyhow::Result;
use futures::StreamExt;
use libp2p::{multiaddr::Protocol, Multiaddr, Swarm};
use log::{error, warn};
use tokio::{
    select,
    sync::mpsc,
    time::{interval, Interval},
};

use crate::{
    behaviour::Behaviour,
    command::{Message, QueryHandler},
    handler::{CommandRequestHandler, EventHandler, Handler},
    lmm::LocalMarketMap,
    BootNodes,
};

pub(super) struct Coordinator {
    query_handler: QueryHandler,
    swarm: Swarm<Behaviour>,
    lmm: LocalMarketMap,
    boot_nodes: Option<BootNodes>,
    command_receiver: mpsc::UnboundedReceiver<Message>,
    bootstrap_interval: Interval,
}

impl Coordinator {
    pub(crate) fn new(
        mut swarm: Swarm<Behaviour>,
        public_address: Option<Multiaddr>,
        boot_nodes: Option<BootNodes>,
        peer_tcp_port: u16,
        command_receiver: mpsc::UnboundedReceiver<Message>,
        bootstrap_time: Duration,
    ) -> Result<Self> {
        let listen_addr = Multiaddr::from(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(peer_tcp_port));
        swarm.listen_on(listen_addr)?;
        let boot_nodes = {
            if let Some(boot_nodes) = boot_nodes {
                for (peer_id, addr) in boot_nodes.get_kad_addrs() {
                    swarm
                        .behaviour_mut()
                        .autonat
                        .add_server(peer_id, Some(addr.clone()));
                    swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                }
                swarm.behaviour_mut().kad.bootstrap()?;
                Some(boot_nodes)
            } else {
                None
            }
        };
        if let Some(public_address) = public_address {
            swarm.add_external_address(public_address);
        }
        Ok(Self {
            boot_nodes,
            lmm: Default::default(),
            query_handler: Default::default(),
            swarm,
            command_receiver,
            bootstrap_interval: interval(bootstrap_time),
        })
    }

    pub(super) async fn run(mut self) {
        loop {
            select! {
                _ = self.bootstrap_interval.tick() => {
                    if let Err(err) = self.swarm.behaviour_mut().kad.bootstrap() {
                        warn!("Failed to bootstrap: {}", err);
                    }
                }
                event = self.swarm.select_next_some() => {
                    let mut handler = Handler::new(&mut self.swarm, &mut self.lmm, &mut self.query_handler, self.boot_nodes.as_ref());
                    handler.handle_event(event);
                }
                command = self.command_receiver.recv() => {
                    if let Some((request, responder)) = command {
                        let mut handler = Handler::new(&mut self.swarm, &mut self.lmm, &mut self.query_handler, self.boot_nodes.as_ref());
                        handler.handle_command(request, responder);
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

impl Drop for Coordinator {
    fn drop(&mut self) {
        error!("Coordinator receiver channel closed");
    }
}
