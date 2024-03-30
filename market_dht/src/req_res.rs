use anyhow::Result;
use libp2p::{Multiaddr, PeerId};
use tokio::sync::oneshot::{self};

pub(crate) type Response = Result<ResponseData>;
pub(crate) type Request = (RequestData, RequestHandler);

#[derive(Debug)]
pub(crate) struct ResponseHandler {
    inner: oneshot::Receiver<Response>,
}

impl ResponseHandler {
    pub(crate) async fn get_response_data(self) -> Response {
        self.inner.await?
    }
}

#[derive(Debug)]
pub(crate) struct RequestHandler {
    inner: oneshot::Sender<Response>,
}

impl RequestHandler {
    pub(crate) fn new() -> (Self, ResponseHandler) {
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
        (
            Self {
                inner: response_sender,
            },
            ResponseHandler {
                inner: response_receiver,
            },
        )
    }

    pub(crate) fn respond(self, response: Response) {
        self.inner
            .send(response)
            .expect("it to send since oneshot client should not have dropped")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub(crate) enum RequestData {
    GetAllListeners,
    GetConnectedPeers,
    IsConnectedTo(PeerId),
    KadRequest(KadRequestData),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ResponseData {
    // NOTE: the vec is useful for now when we add functionality for users being able to add
    // listeners?
    AllListeners { listeners: Vec<Multiaddr> },
    ConnectedPeers { connected_peers: Vec<PeerId> },
    IsConnectedTo { is_connected: bool },
    KadResponse(KadResponseData),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum KadRequestData {
    ClosestLocalPeers { key: Vec<u8> },
    ClosestPeers { key: Vec<u8> },
    GetFile { key: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KadResponseData {
    ClosestLocalPeers { peers: Vec<PeerId> },
    ClosestPeers { key: Vec<u8>, peers: Vec<PeerId> },
    GetFile { peer: Option<PeerId> },
}
