use crate::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

use self::proto::FileHolders;

pub trait DhtEntry: Serialize + DeserializeOwned {
    fn key_namespace() -> &'static str;
    // combine old record and new record
    // perform validation here
    fn update(key: &[u8], cur: Self, new: Self) -> Self;
}

impl DhtEntry for FileHolders {
    fn key_namespace() -> &'static str {
        "HoldersResponse"
    }
    fn update(key: &[u8], cur: FileHolders, new_values: FileHolders) -> FileHolders {
        let key_str = std::str::from_utf8(key).unwrap();
        /*
        // check that the key is a valid sha256 hash (right now, leave it out to make testing with the test_client easier)
        if key_str.len() != 64 {
            return false;
        }
        */

        if key_str != new_values.file_info.file_hash {
            println!("File hash does not match key");
            return cur;
        }
        if cur.file_info != new_values.file_info {
            println!("File info does not match current");
            return cur;
        }

        let mut merged_ids: HashMap<String, (User, u64)> = cur
            .holders
            .iter()
            .map(|x| (x.0.id.clone(), x.clone()))
            .collect();

        let now = get_current_time();

        // merge new and old requests
        for (holder, expiration) in new_values.holders {
            // check that the expiration date is valid
            if expiration < now || expiration > now + EXPIRATION_OFFSET {
                println!("Invalid expiration");
                continue;
            }

            // if there are duplicates, just keep the furthest valid expiration time
            if let Some((existing_user, existing_expiration)) = merged_ids.get_mut(&holder.id) {
                // keep the longest expiration date that is within the expiration limit
                let expiration = std::cmp::min(
                    std::cmp::max(expiration, *existing_expiration),
                    now + EXPIRATION_OFFSET,
                );
                *existing_user = holder;
                *existing_expiration = expiration;
            } else {
                merged_ids.insert(holder.id.clone(), (holder, expiration));
            }
        }

        FileHolders {
            holders: merged_ids.into_values().collect(),
            ..cur
        }
    }
}

// set of all file hashes being provided
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ProvidedFiles(pub HashSet<String>);

impl DhtEntry for ProvidedFiles {
    fn key_namespace() -> &'static str {
        "ProvidedFiles"
    }
    fn update(_key: &[u8], cur: ProvidedFiles, new_values: ProvidedFiles) -> ProvidedFiles {
        ProvidedFiles(&cur.0 | &new_values.0)
    }
}
