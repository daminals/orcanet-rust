// TODO: Add tests for the producer module

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::collections::HashMap;
//     use std::path::PathBuf;
//     use tokio::runtime::Runtime;

//     #[tokio::test]
//     async fn test_start_server() {
//         let files: HashMap<String, PathBuf> = HashMap::new();
//         let prices: HashMap<String, i64> = HashMap::new();
//         let port = "8080".to_string();

//         let handle = start_server(files, prices, port).await;

//         // Wait for the server to start
//         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

//         // Perform assertions or additional tests here``
//         assert!(handle.is_alive());

//         // Stop the server
//         handle.abort();
//     }
// }
