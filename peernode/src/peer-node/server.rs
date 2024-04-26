pub mod consumer;
pub mod grpc;
pub mod producer;
pub mod store;

use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

// shared server state
#[derive(Clone)]
pub struct ServerState {
    pub config: Arc<Mutex<store::Configurations>>,
}

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
    price: i64,
}
// UploadFile - To upload a file. This endpoint should accept a file (likely in Base64) and handle the storage and processing of the file on the server. Returns the file hash.
// For Now, upload a file path?
async fn upload_file(
    State(state): State<ServerState>,
    Json(file): Json<UploadFile>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;

    let hash = config.add_file(file.filePath, file.price);

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

#[derive(Deserialize)]
struct AddJob {
    fileHash: String,
    peerId: String,
}
// AddJob - Adds a job to the producer's job queue
// takes in a fileHash and peerID.
// returns a jobId of the newly created job
async fn add_job(State(state): State<ServerState>, Json(job): Json<AddJob>) -> impl IntoResponse {
    let config = state.config.lock().await;

    let file_hash = job.fileHash;
    let peer_id = job.peerId;

    let file_name = match config.get_file_names().get(&file_hash) {
        Some(filename) => filename.clone(),
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!("{{\"error\": \"File not found\"}}")))
                .unwrap();
        }
    };

    let file_size = config.get_file_size(file_name.clone());

    let job_id = config
        .get_jobs_state()
        .add_job(file_hash.clone(), file_size, file_name, peer_id.clone())
        .await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("{{\"jobId\": \"{}\"}}", job_id)))
        .unwrap()
}

// Get Job - Adds a job to the producer's job queue
// returns a list of jobs
async fn get_job_list(State(state): State<ServerState>) -> impl IntoResponse {
    let config = state.config.lock().await;
    let jobs_list = config.get_jobs_state().get_jobs_list().await;

    let str_list = serde_json::to_string(&jobs_list).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("{{\"jobs\": \"{:?}\"}}", str_list)))
        .unwrap()
}

// Main function to setup and run the server
#[tokio::main]
async fn main() {
    let config = store::Configurations::new().await;
    let state = ServerState {
        config: Arc::new(Mutex::new(config)),
    };

    let app = Router::new()
        //.route("/file/:hash", get(get_file))
        .route("/upload", post(upload_file))
        .route("/file/:hash/info", get(get_file_info))
        .route("/file/:hash", post(delete_file))
        .route("/add-job", put(add_job))
        .route("/job-list", get(get_job_list))
        .with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
