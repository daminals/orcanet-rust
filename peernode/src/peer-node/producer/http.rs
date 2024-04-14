use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::wallet::AsyncWallet;

use super::files::AsyncFileMap;
use super::state::AppState;

#[derive(Deserialize, Debug)]
struct FileParams {
    chunk: Option<u64>,
}

#[axum::debug_handler]
async fn handle_file_request(
    params: Path<String>,
    query: Query<FileParams>,
    state: State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    // Obtain file hash, chunk, and consumer address
    let hash = params.0;
    let chunk = query.chunk.unwrap_or(0);

    // Parse the Authorization header
    let mut auth_token = if let Some(auth) = headers.get("Authorization") {
        auth.to_str().unwrap_or_default()
    } else {
        ""
    };

    // Remove the "Bearer " prefix
    if !auth_token.is_empty() && auth_token.starts_with("Bearer ") {
        auth_token = &auth_token[7..];
    }

    // Verify the consumer's payment for this chunk
    match state.verify_payment(&hash, auth_token, chunk).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to verify payment: {:?}", e);
            return (StatusCode::FORBIDDEN, "Payment verification failed").into_response();
        }
    }

    // Get the file and its name
    let file = match state.get_file_access(&hash).await {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to get file access: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };
    let file_name = match file.get_name() {
        Ok(file_name) => file_name,
        Err(e) => {
            eprintln!("Failed to get file name: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    // Get the desired chunk
    let file_chunk: Vec<u8> = match file.get_chunk(chunk).await {
        Ok(file_chunk) => match file_chunk {
            Some(file_chunk) => file_chunk,
            None => {
                println!(
                    "HTTP: Chunk [{}] from {} out of range, sending 404",
                    chunk, file_name
                );
                return (
                    StatusCode::NOT_FOUND,
                    format!("Chunk [{}] not found", chunk),
                )
                    .into_response();
            }
        },
        Err(e) => {
            eprintln!("Failed to get chunk {} from {}: {}", chunk, file_name, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    // Create a stream from the file chunk
    let body = Body::from(file_chunk);

    // Get the content type using mime_guess
    let mime = mime_guess::from_path(&file_name).first_or_octet_stream();

    println!(
        "HTTP: Sending Chunk [{}] for file {:?} to consumer {}",
        chunk, file_name, auth_token
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.to_string())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_name),
        )
        .body(body)
        .unwrap()
}

#[axum::debug_handler]
async fn handle_invoice_request(params: Path<String>, state: State<Arc<AppState>>) -> Response {
    // Obtain file hash
    let hash = params.0;

    // Generate an invoice for the file
    let consumer = match state.generate_invoice(&hash).await {
        Ok(consumer) => consumer,
        Err(e) => {
            eprintln!("Failed to generate invoice: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    // Return the invoice and token
    let invoice = consumer.invoice;
    let token = consumer.token;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .header("X-Access-Token", token)
        .body(Body::from(invoice))
        .unwrap()
}

pub async fn run(
    files: AsyncFileMap,
    wallet: AsyncWallet,
    port: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("HTTP: Listening on {}", listener.local_addr()?);

    // Create state
    let state = AppState::new(wallet, files);

    let app = Router::new()
        .route("/file/:file_hash", get(handle_file_request))
        .route("/invoice/:file_hash", get(handle_invoice_request))
        .with_state(Arc::new(state));

    axum::serve(listener, app).await?;
    Ok(())
}
