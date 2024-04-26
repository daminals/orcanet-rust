pub mod consumer;
pub mod grpc;
pub mod producer;
pub mod store;

mod routes {
    pub mod file_routes;
    pub mod job_routes;
}

use axum::Router;
use std::sync::Arc;
use tokio::sync::Mutex;

// shared server state
#[derive(Clone)]
pub struct ServerState {
    pub config: Arc<Mutex<store::Configurations>>,
}

// Main function to setup and run the server
#[tokio::main]
async fn main() {
    let config = store::Configurations::new().await;
    let state = ServerState {
        config: Arc::new(Mutex::new(config)),
    };

    let app = Router::new()
        .merge(routes::file_routes::routes())
        .merge(routes::job_routes::routes())
        .with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
