use crate::market::User;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

// Encode a user struct to a base64 string
pub fn encode_user(user: &User) -> String {
    let user_str = serde_json::to_string(&user).unwrap();
    let encoded_user = general_purpose::STANDARD.encode(user_str.as_bytes());
    encoded_user
}

// Decode a base64 string to a user struct
pub fn decode_user(encoded_user: String) -> Result<User> {
    let user_str = match general_purpose::STANDARD.decode(encoded_user) {
        Ok(user_str) => match String::from_utf8(user_str) {
            Ok(user_str) => user_str,
            Err(_) => {
                return Err(anyhow::anyhow!("Failed to decode user"));
            }
        },
        Err(_) => {
            return Err(anyhow::anyhow!("Failed to decode user"));
        }
    };

    match serde_json::from_str(&user_str) {
        Ok(user) => Ok(user),
        Err(_) => Err(anyhow::anyhow!("Failed to parse user")),
    }
}
