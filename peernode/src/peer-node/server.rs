pub mod consumer;
pub mod grpc;
pub mod producer;
pub mod store;

use axum::{
    body::Body,
    extract::{Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct FileParams {
    chunk: Option<u64>,
    producer: String,
    continue_download: bool,
}

// GetFile - Fetches a file from a given hash/CID.
async fn get_file(
    // Path(hash): Path<String>,
    params: Path<String>,
    query: Query<FileParams>,
) -> impl IntoResponse {
    let mut config = store::Configurations::new().await;
    let hash = params.0;
    let producer = query.producer.clone();
    let continue_download = query.continue_download;
    let token = config.get_token(producer.to_string());
    let chunk_num = query.chunk.unwrap_or(0);

    let ret_token = match consumer::get_file(
        producer.to_string(),
        hash.clone(),
        token.clone(),
        chunk_num,
        continue_download,
    )
    .await
    {
        Ok(new_token) => new_token,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    // Update the token in configurations
    config.set_token(producer.to_string(), ret_token.clone());

    // Build and return the response
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(
            "{{\"hash\": \"{}\", \"token\": \"{}\"}}",
            hash, ret_token
        )))
        .unwrap()
}

// GetFileInfo - Fetches files info from a given hash/CID. Should return name, size, # of peers, whatever other info you can give.
async fn get_file_info(query: Query<FileParams>) -> impl IntoResponse {
    let mut config = store::Configurations::new().await;
    let producer = query.producer.clone();
    let market_client = match config.get_market_client().await {
        Ok(client) => client,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };
    let ret_token = match consumer::list_producers(producer.to_string(), market_client).await {
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

// UploadFile - To upload a file. This endpoint should accept a file (likely in Base64) and handle the storage and processing of the file on the server. Returns the file hash.
// async fn upload_file(Json(file): Json<producer::files::File>) -> impl IntoResponse {
// TODO: Implement this function-- we still do not know how to handle file uploads
// }

// DeleteFile - Deletes a file from the configurations (deleting from server not possible)
async fn delete_file(Path(hash): Path<String>) -> impl IntoResponse {
    let mut config = store::Configurations::new().await;
    config.remove_file(hash.clone());
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("{{\"hash\": \"{}\"}}", hash)))
        .unwrap()

}


// Main function to setup and run the server
#[tokio::main]
async fn main() {
    let _app = Router::<StatusCode>::new()
        .route("/file/:hash", get(get_file))
        .route("/file/:hash/info", get(get_file_info))
        .route("/file/:hash", post(delete_file));
}

