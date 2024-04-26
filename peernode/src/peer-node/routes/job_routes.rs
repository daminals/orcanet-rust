use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;

use crate::ServerState;

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
        Some(file_name) => file_name.clone(),
        None => {
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };
    let price = match config.get_prices().get(&file_hash) {
        Some(price) => price.clone(),
        None => {
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };
    let file_size = config.get_file_size(file_name.clone());

    let job_id = config
        .get_jobs_state()
        .add_job(
            file_hash.clone(),
            file_size,
            file_name,
            price.clone(),
            peer_id.clone(),
        )
        .await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("{{\"jobID\": \"{}\"}}", job_id)))
        .unwrap()
}

// Get Job - Adds a job to the producer's job queue
// returns a list of jobs
async fn get_job_list(State(state): State<ServerState>) -> impl IntoResponse {
    let config = state.config.lock().await;
    let jobs_list = config.get_jobs_state().get_jobs_list().await;

    let jobs_json = serde_json::to_string(&jobs_list).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("{{\"jobs\": \"{:?}\"}}", jobs_json)))
        .unwrap()
}

// Get Job Info
async fn get_job_info(
    State(state): State<ServerState>,
    Path(jobID): Path<String>,
) -> impl IntoResponse {
    let config = state.config.lock().await;

    let job_info = match config.get_jobs_state().get_job_info(&jobID).await {
        Some(job_info) => job_info,
        None => {
            return (StatusCode::NOT_FOUND, "Job not found").into_response();
        }
    };

    let info_json = serde_json::to_string(&job_info).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(info_json))
        .unwrap()
}

// Get History
async fn get_history(State(state): State<ServerState>) -> impl IntoResponse {
    let config = state.config.lock().await;
    let history = config.get_jobs_state().get_job_history().await;

    let history_json = serde_json::to_string(&history).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("{{\"jobs\": \"{:?}\"}}", history_json)))
        .unwrap()
}

#[derive(Deserialize)]
struct RemoveFromHistory {
    jobID: String,
}
// Remove From History
async fn remove_from_history(
    State(state): State<ServerState>,
    Json(job): Json<RemoveFromHistory>,
) -> impl IntoResponse {
    let config = state.config.lock().await;

    let successful = config
        .get_jobs_state()
        .remove_job_from_history(&job.jobID)
        .await;

    if !successful {
        return (StatusCode::NOT_FOUND, "Job not found").into_response();
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Job successfully removed from history"))
        .unwrap()
}

// Clear History
async fn clear_history(State(state): State<ServerState>) -> impl IntoResponse {
    let config = state.config.lock().await;
    config.get_jobs_state().clear_job_history().await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("History cleared"))
        .unwrap()
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/get-history", get(get_history))
        .route("/remove-from-history", put(remove_from_history))
        .route("/clear-history", put(clear_history))
        .route("/add-job", put(add_job))
        .route("/job-list", get(get_job_list))
        .route("/job-info/:jobID", get(get_job_info))
}
