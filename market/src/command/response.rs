use libp2p::{Multiaddr, PeerId};
use proto::market::HoldersResponse;
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

use crate::lmm::FileResponse;

pub type Response = Result<SuccessfulResponse, FailureResponse>;

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SuccessfulResponse {
    Listeners { listeners: Vec<Multiaddr> },
    ConnectedPeers { peers: Vec<PeerId> },
    ConnectedTo { connected: bool },
    CheckHolders(HoldersResponse),
    KadResponse(KadSuccessfulResponse),
    LmmResponse(LmmSuccessfulResponse),
    ReqResResponse(ReqResSuccessfulResponse),
}

#[derive(Debug, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum FailureResponse {
    #[error("Failed to send request: {0}")]
    SendError(String),
    #[error("Failed to receive response: {0}")]
    RecvError(#[from] RecvError),
    #[error("[Kademlia Error] - {0}")]
    KadError(KadFailureResponse),
    #[error("[Local Market Map Error] - {0}")]
    LmmError(LmmFailureResponse),
    #[error("[Request Response Error] - {0}")]
    ReqResError(ReqResFailureResponse),
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LmmSuccessfulResponse {
    IsLocalFileOwner { is_owner: bool },
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KadSuccessfulResponse {
    GetClosestPeers { peers: Vec<PeerId> },
    RegisterFile,
    GetProviders { providers: Vec<PeerId> },
}

#[derive(Debug, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum KadFailureResponse {
    #[error("Failed to get closest peers: {error}")]
    GetClosestPeers { key: Vec<u8>, error: String },
    #[error("Failed to register file: {error}")]
    RegisterFile { error: String },
    #[error("Failed to get providers: {error}")]
    GetProviders { error: String },
}

#[derive(Debug, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum LmmFailureResponse {}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReqResSuccessfulResponse {
    GetHolderByPeerId { holder: FileResponse },
}

#[derive(Debug, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ReqResFailureResponse {
    #[error("Failed to get holder by peer id: {error}")]
    GetHolderByPeerId { error: String },
}
