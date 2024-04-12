use serde::{de::DeserializeOwned, Serialize};

use crate::{get_current_time, EXPIRATION_OFFSET};
use std::collections::HashMap;

use crate::FileRequest;

pub trait DhtEntry: Serialize + DeserializeOwned {
    fn key_namespace() -> &'static str;
    // combine old record and new record
    // perform validation here
    fn update(key: &[u8], cur: Self, new: Self) -> Self;
}

impl DhtEntry for Vec<FileRequest> {
    fn key_namespace() -> &'static str {
        "Vec<FileRequest>"
    }
    fn update(key: &[u8], cur: Vec<FileRequest>, new_values: Vec<FileRequest>) -> Vec<FileRequest> {
        let key_str = std::str::from_utf8(key).unwrap();
        /*
        // check that the key is a valid sha256 hash (right now, leave it out to make testing with the test_client easier)
        if key_str.len() != 64 {
            return false;
        }
        */

        let mut merged_ids: HashMap<String, FileRequest> =
            cur.iter().map(|x| (x.user.id.clone(), x.clone())).collect();

        let now = get_current_time();

        // merge new and old requests
        for new in new_values {
            // check that the expiration date is valid
            if new.expiration < now || new.expiration > now + EXPIRATION_OFFSET {
                println!("Invalid expiration");
                continue;
            }

            if key_str != new.file_hash {
                println!("File hash does not match key");
                continue;
            }

            // if there are duplicates, just keep the furthest valid expiration time
            if let Some(existing) = merged_ids.get_mut(&new.user.id) {
                // keep the longest expiration date that is within the expiration limit
                let expiration = std::cmp::min(
                    std::cmp::max(new.expiration, existing.expiration),
                    now + EXPIRATION_OFFSET,
                );
                existing.expiration = expiration;
            } else {
                merged_ids.insert(new.user.id.clone(), new);
            }
        }

        merged_ids.into_values().collect()
    }
}
