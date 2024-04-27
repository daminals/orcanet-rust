#![allow(non_snake_case)]
use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{consumer, producer, ServerState};

#[derive(Deserialize, Debug)]
struct FileParams {
    chunk: String,
    producer: String,
    continue_download: String,
}

#[derive(Serialize)]
struct FindPeerRet {
    peers: Vec<Peer>,
}
#[derive(Serialize)]
struct Peer {
    peerId: String,
    ip: String,
    region: String,
    price: f64,
}
// Finds all peers hosting a given file
async fn find_peer(
    State(state): State<ServerState>,
    Path(fileHash): Path<String>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;

    let market_client = match config.get_market_client().await {
        Ok(client) => client,
        Err(_) => {
            eprintln!("No market client initialized");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };
    match market_client.check_holders(fileHash).await {
        Ok(res) => {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&FindPeerRet {
                        peers: res
                            .into_iter()
                            .map(|user| Peer {
                                peerId: user.username,
                                ip: user.ip.to_string(),
                                region: "US".into(),
                                price: user.price as f64,
                            })
                            .collect(),
                    })
                    .unwrap(),
                ))
                .unwrap()
        }
        _ => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    }
}

// GetFileInfo - Fetches files info from a given hash/CID. Should return name, size, # of peers, whatever other info you can give.
// TODO: update to the new spec on the doc
async fn get_file_info(
    State(state): State<ServerState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;

    let producer = "this arg was removed"; //query.producer.clone()";
    let market_client = match config.get_market_client().await {
        Ok(client) => client,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    let ret_token = match consumer::list_producers(hash, market_client).await {
        Ok(new_token) => new_token,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };
    config.set_token(producer.to_string(), ret_token.clone());

    // Build and return the response
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(
            "{{\"producer list\": \"{}\", \"token\": \"{}\"}}",
            producer, ret_token
        )))
        .unwrap()
}

#[derive(Deserialize)]
struct UploadFile {
    filePath: String,
}
// UploadFile - To upload a file. This endpoint should accept a file (likely in Base64) and handle the storage and processing of the file on the server. Returns the file hash.
// For Now, upload a file path?
async fn upload_file(
    State(state): State<ServerState>,
    Json(file): Json<UploadFile>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;

    // TODO: fetch the price from the config somehow, likely from somewhere not yet implemented
    let price = 416;

    let hash = config.add_file(file.filePath, price);


    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("{{\"hash\": \"{}\"}}", hash)))
        .unwrap()
}

// DeleteFile - Deletes a file from the configurations
async fn delete_file(
    State(state): State<ServerState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;
    config.remove_file(hash.clone());

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("{{\"hash\": \"{}\"}}", hash)))
        .unwrap()
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        // [Bubble Guppies]
        // ## Market Page
        .route("/find-peer/:fileHash", get(find_peer))
        //.route("/file/:hash", get(get_file))
        .route("/upload", post(upload_file))
        .route("/file/:hash/info", get(get_file_info))
        .route("/file/:hash", delete(delete_file))
}