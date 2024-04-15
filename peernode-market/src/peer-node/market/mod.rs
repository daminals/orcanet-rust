pub mod dht;
pub mod dht_entry;
pub mod market;

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// time in secs that a file is valid for
pub const EXPIRATION_OFFSET: u64 = 3600;

// get the current time in seconds
pub fn get_current_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub price: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldersResponse {
    pub holders: Vec<User>,
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRequest {
    pub user: User,
    pub file_hash: String,
    pub expiration: u64,
}
