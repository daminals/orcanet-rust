use libp2p::{swarm::SwarmEvent, Swarm};
use log::error;
use tokio::sync::oneshot;

use crate::{
    behaviour::Behaviour,
    command::{request::Request, QueryHandler},
    lmm::LocalMarketMap,
    BootNodes, Response, SuccessfulResponse,
};

use self::{
    autonat::AutoNatHandler,
    dcutr::DcutrHandler,
    identify::IdentifyHandler,
    kad::KadHandler,
    ping::PingHandler,
    relay::{client::RelayClientHandler, server::RelayServerHandler},
};

use super::behaviour::BehaviourEvent;

pub(crate) trait EventHandler {
    type Event;
    fn handle_event(&mut self, event: Self::Event);
}

pub(crate) trait CommandRequestHandler {
    fn handle_command(&mut self, request: Request, responder: oneshot::Sender<Response>);
}

pub(crate) struct Handler<'a> {
    swarm: &'a mut Swarm<Behaviour>,
    lmm: &'a mut LocalMarketMap,
    query_handler: &'a mut QueryHandler,
    boot_nodes: Option<&'a BootNodes>,
}

impl<'a> Handler<'a> {
    pub(crate) fn new(
        swarm: &'a mut Swarm<Behaviour>,
        lmm: &'a mut LocalMarketMap,
        query_handler: &'a mut QueryHandler,
        boot_nodes: Option<&'a BootNodes>,
    ) -> Self {
        Handler {
            swarm,
            lmm,
            query_handler,
            boot_nodes,
        }
    }
}

impl<'a> EventHandler for Handler<'a> {
    type Event = SwarmEvent<BehaviourEvent>;

    fn handle_event(&mut self, event: Self::Event) {
        // NOTE:  maybe use  box,dyn but that would remove zca?
        // or implement a proc macro in the future
        match event {
            SwarmEvent::Behaviour(event) => match event {
                BehaviourEvent::Kad(event) => {
                    let mut kad_handler = KadHandler::new(self.swarm, self.lmm, self.query_handler);
                    kad_handler.handle_event(event);
                }
                BehaviourEvent::Identify(event) => {
                    let mut identify_handler = IdentifyHandler::new(self.swarm);
                    identify_handler.handle_event(event);
                }
                BehaviourEvent::Ping(event) => {
                    let mut ping_handler = PingHandler {};
                    ping_handler.handle_event(event);
                }
                BehaviourEvent::Autonat(event) => {
                    let mut autonat_handler = AutoNatHandler::new(self.boot_nodes);
                    autonat_handler.handle_event(event);
                }
                BehaviourEvent::RelayServer(event) => {
                    let mut relay_server_handler = RelayServerHandler {};
                    relay_server_handler.handle_event(event);
                }
                BehaviourEvent::Dcutr(event) => {
                    let mut dcutr_handler = DcutrHandler {};
                    dcutr_handler.handle_event(event);
                }
                BehaviourEvent::RelayClient(event) => {
                    let mut relay_client = RelayClientHandler {};
                    relay_client.handle_event(event);
                }
            },
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in,
            } => todo!(),
            SwarmEvent::ConnectionClosed {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                cause,
            } => todo!(),
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => todo!(),
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
            } => todo!(),
            SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
            } => todo!(),
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => todo!(),
            SwarmEvent::ExpiredListenAddr {
                listener_id,
                address,
            } => todo!(),
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => todo!(),
            SwarmEvent::ListenerError { listener_id, error } => todo!(),
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => todo!(),
            SwarmEvent::NewExternalAddrCandidate { address } => todo!(),
            SwarmEvent::ExternalAddrConfirmed { address } => todo!(),
            SwarmEvent::ExternalAddrExpired { address } => todo!(),
            _ => {}
        }
    }
}

impl<'a> CommandRequestHandler for Handler<'a> {
    fn handle_command(&mut self, request: Request, responder: oneshot::Sender<Response>) {
        match request {
            Request::Listeners => {
                let listeners = self.swarm.listeners().cloned().collect();
                send_ok!(responder, SuccessfulResponse::Listeners { listeners });
            }
            Request::ConnectedPeers => {
                let peers = self.swarm.connected_peers().cloned().collect();
                send_ok!(responder, SuccessfulResponse::ConnectedPeers { peers });
            }
            Request::ConnectedTo { peer_id } => {
                let connected = self.swarm.is_connected(&peer_id);
                send_ok!(responder, SuccessfulResponse::ConnectedTo { connected });
            }
        };
    }
}

mod macros {
    macro_rules! send_ok {
        ($sender:expr, $response:expr) => {
            if let Err(_) = $sender.send(Ok($response)) {
                error!("Failed to send response back to peer!");
            }
        };
    }
    macro_rules! send_err {
        ($sender:expr, $response:expr) => {
            if let Err(_) = $sender.send(Err($response)) {
                error!("Failed to send response back to peer!");
            }
        };
    }
    pub(crate) use send_err;
    pub(crate) use send_ok;
}
pub(crate) use macros::send_err;
pub(crate) use macros::send_ok;

mod autonat;
mod dcutr;
mod identify;
mod kad;
mod ping;
mod relay;
