use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{consumer, ServerState};

#[derive(Deserialize, Debug)]
struct FileParams {
    chunk: String,
    producer: String,
    continue_download: String,
}

// This endpoint was removed in the latest API
//
// async fn get_file(
//     // Path(hash): Path<String>,
//     State(state): State<ServerState>,
// ) -> Result<impl IntoResponse, &'static str> {
//     let mut config = state.config.lock().await.unwrap();
//     let hash = params.0;
//     let producer = query.producer.clone();
//     let continue_download = match query.continue_download.clone().to_lowercase().as_str() {
//         "true" => true,
//         "false" => false,
//         _ => {
//             // Return an error if the string is neither "true" nor "false"
//             return Err("Invalid value for continue_download");
//         }
//     };
//     let token = config.get_token(producer.to_string());
//     let chunk_num = match query.chunk.clone().parse::<u64>() {
//         Ok(chunk_num) => chunk_num,
//         Err(_) => {
//             // Return an error if parsing fails
//             return Err("Invalid chunk number");
//         }
//     };

//     let ret_token = match consumer::get_file(
//         producer.to_string(),
//         hash.clone(),
//         token.clone(),
//         chunk_num,
//         continue_download,
//     )
//     .await
//     {
//         Ok(new_token) => new_token,
//         Err(_) => {
//             return Ok((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response());
//         }
//     };

//     // Update the token in configurations
//     config.set_token(producer.to_string(), ret_token.clone());

//     // Build and return the response
//     Ok(Response::builder()
//         .status(StatusCode::OK)
//         .header(header::CONTENT_TYPE, "application/json")
//         .body(Body::from(format!(
//             "{{\"hash\": \"{}\", \"token\": \"{}\"}}",
//             hash, ret_token
//         )))
//         .unwrap())
// }

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
        //.route("/file/:hash", get(get_file))
        .route("/upload", post(upload_file))
        .route("/file/:hash/info", get(get_file_info))
        .route("/file/:hash", post(delete_file))
}
