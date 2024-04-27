#![allow(non_snake_case)]
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

use crate::{consumer::encode, producer, ServerState};

#[derive(Deserialize)]
struct AddJob {
    fileHash: String,
    peerId: String,
}
// AddJob - Adds a job to the producer's job queue
// takes in a fileHash and peerID.
// returns a jobId of the newly created job
async fn add_job(State(state): State<ServerState>, Json(job): Json<AddJob>) -> impl IntoResponse {
    let mut config = state.config.lock().await;

    let file_hash = job.fileHash;
    let peer_id = job.peerId;

    let file_info;
    let user = match config.get_market_client().await {
        Ok(market) => {
            match market.check_holders(file_hash.clone()).await {
                Ok(Some(res)) => {
                    file_info = res.file_info.unwrap();
                    res.holders.into_iter().filter(|user| user.id == peer_id).next()
                }
                _ => return (StatusCode::SERVICE_UNAVAILABLE, "Could not check holders").into_response(),
            }
        }
        Err(_) => return (StatusCode::SERVICE_UNAVAILABLE, "Market not available").into_response(),
    };
    let user = match user {
        Some(user) => user,
        None => return (StatusCode::NOT_FOUND, "Peer is not providing file").into_response(),
    };
    let encoded_producer = encode::encode_user(&user);
    println!("Encoded producer: {encoded_producer}");
    println!("id: {peer_id}");
    let job_id = config
        .get_jobs_state()
        .add_job(
            file_info.file_hash,
            file_info.file_size as u64,
            file_info.file_name,
            user.price,
            peer_id.clone(),
            encoded_producer,
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
#[allow(non_snake_case)]
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

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct StartJobIds {
    jobIDs: Vec<String>,
}
// Start Jobs
async fn start_jobs(
    State(state): State<ServerState>,
    Json(arg): Json<StartJobIds>,
) -> impl IntoResponse {
    for job_id in arg.jobIDs {
        let mut config = state.config.lock().await;
        match config.get_jobs_state().get_job(&job_id).await {
            Some(job) => {
                let token = config.get_token(job.lock().await.encoded_producer.clone());
                producer::jobs::start(job, token).await;
            }
            None => return (StatusCode::NOT_FOUND, "Job not found").into_response(),
        }
    }
    StatusCode::OK.into_response()
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        // [Bubble Guppies]
        // ## Market Page
        .route("/get-history", get(get_history))
        .route("/remove-from-history", put(remove_from_history))
        .route("/clear-history", put(clear_history))
        .route("/add-job", put(add_job))
        .route("/job-list", get(get_job_list))
        .route("/job-info/:jobID", get(get_job_info))
        .route("/start-jobs", put(start_jobs))
}
