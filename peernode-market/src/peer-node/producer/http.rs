use axum::{
    body::Body,
    extract::{ConnectInfo, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use crate::producer::db;

use super::files::AsyncFileMap;
use super::files::FileAccessType;

#[derive(Clone)]
struct AppState {
    consumers: Arc<db::Consumers>,
    files: AsyncFileMap,
}

#[derive(Deserialize, Debug)]
struct FileParams {
    chunk: Option<u64>,
}

#[axum::debug_handler]
async fn handle_file_request(
    params: Path<String>,
    query: Query<FileParams>,
    state: State<AppState>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    // Obtain file hash, chunk, and consumer address
    let hash = params.0;
    let chunk = query.chunk.unwrap_or(0);
    let address = connect_info.0.ip().to_string();

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

    // Get the consumer
    let consumer = match state.consumers.get_consumer(&address).await {
        Some(consumer) => consumer,
        None => {
            // Create a new consumer
            let consumer = db::Consumer {
                wallet_address: "wallet_address".to_string(),
                requests: HashMap::new(),
            };

            state
                .consumers
                .add_consumer(address.clone(), consumer)
                .await
        }
    };
    let mut consumer = consumer.lock().await;

    // Get the consumer request
    let request = match consumer.requests.get_mut(&hash) {
        Some(request) => request,
        None => {
            // Create a new consumer request
            let request = db::ConsumerRequest {
                chunks_sent: 0,
                access_token: "".to_string(),
            };

            consumer.requests.insert(hash.clone(), request);
            consumer.requests.get_mut(&hash).unwrap()
        }
    };

    // Check if an access token is expected (first chunk is "free")
    if request.chunks_sent == 0 {
        request.access_token = db::generate_access_token();
    } else if request.access_token != auth_token {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    // Get the file path from the file map
    let file_path = match state.files.get_file_path(&hash).await {
        Some(path) => path,
        None => {
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };
    // Get the file name
    let file_name = match file_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            eprintln!("Failed to get file name from {:?}", file_path);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    // Create a new FileAccessType, which will open the file and allow us to read chunks
    let file = match FileAccessType::new(&file_path.to_string_lossy().to_string()) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Failed to open file {:?}", file_path);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    // Get the desired chunk
    let file_chunk: Vec<u8> = match file.get_chunk(chunk).await {
        Ok(file_chunk) => match file_chunk {
            Some(file_chunk) => file_chunk,
            None => {
                println!(
                    "HTTP: Chunk [{}] from {:?} out of range, sending 404",
                    chunk, file_path
                );
                return (
                    StatusCode::NOT_FOUND,
                    format!("Chunk [{}] not found", chunk),
                )
                    .into_response();
            }
        },
        Err(e) => {
            eprintln!(
                "Failed to get chunk {} from {:?}: {:?}",
                chunk, file_path, e
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response();
        }
    };

    // Create a stream from the file chunk
    let body = Body::from(file_chunk);

    // Get the content type using mime_guess
    let mime = mime_guess::from_path(&file_name).first_or_octet_stream();

    // Increment the chunks sent
    request.chunks_sent += 1;

    println!(
        "HTTP: Sending Chunk [{}] for file {:?} to consumer {}",
        chunk, file_path, address
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.to_string())
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_name),
        )
        .header("X-Access-Token", request.access_token.as_str())
        .body(body)
        .unwrap()
}

pub async fn run(files: AsyncFileMap, port: String) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("HTTP: Listening on {}", listener.local_addr()?);

    let app = Router::new()
        .route("/file/:file_hash", get(handle_file_request))
        .with_state(AppState {
            consumers: Arc::new(db::Consumers::new()),
            files,
        });

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
