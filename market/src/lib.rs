pub mod dht;
pub mod dht_entry;
pub mod proto;

use crate::dht::*;

use std::io::Write;

use ::proto::market::{FileInfo, HoldersResponse, User};
use anyhow::{anyhow, Context, Result};
use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use proto::FileHolders;
use tokio::task::JoinHandle;

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

#[derive(Debug)]
pub struct Market {
    dht_client: DhtClient,
    dht_handle: JoinHandle<()>,
}

impl Market {
    pub async fn new(
        bootstrap_peers: &[Multiaddr],
        private_key: Option<String>,
        listen_address: Option<Multiaddr>,
    ) -> Result<Self> {
        let id_keys = if let Some(private_key) = private_key {
            let mut bytes = std::fs::read(&private_key)
                .with_context(|| format!("Failed to read private key '{private_key}' bytes"))?;
            let id_keys = Keypair::rsa_from_pkcs8(&mut bytes)
                .with_context(|| format!("Failed to decode private key '{private_key}'"))?;
            println!("Peer Id: {}", id_keys.public().to_peer_id());
            let mut peer_id_file = std::fs::File::create("peer_id.txt")?;
            peer_id_file.write_all(format!("{}", id_keys.public().to_peer_id()).as_bytes())?;
            Some(id_keys)
        } else {
            None
        };

        let listen_on = listen_address.zip(id_keys);

        let (dht_client, dht_handle) =
            match DhtClient::spawn_client(bootstrap_peers, listen_on).await {
                Ok(o) => o,
                Err(err) => return Err(anyhow!("{err}")),
            };
        //dht_handle.await?

        Ok(Self {
            dht_client,
            dht_handle,
        })
    }

    // Register a new producer
    pub async fn register_file(
        &mut self,
        id: String,
        name: String,
        ip: String,
        port: i32,
        price: i64,
        file_info: FileInfo,
    ) -> Result<()> {
        let user = User {
            id,
            name,
            ip,
            port,
            price,
        };

        // insert the file request into the market data and validate the holders
        self.insert_and_validate(user, file_info, get_current_time() + EXPIRATION_OFFSET)
            .await;

        Ok(())
    }

    // Get a list of producers for a given file hash
    pub async fn check_holders(&self, file_hash: String) -> Result<Option<HoldersResponse>> {
        let now = get_current_time();

        let mut users = vec![];

        let mut holders = match self.dht_client.get_holders(file_hash.as_str()).await? {
            Some(holders) => holders,
            None => return Ok(None),
        };

        // check if any of the files have expired

        let mut first_valid = -1;
        //TODO: use binary search since times are inserted in order
        for (i, holder) in holders.holders.iter().enumerate() {
            if holder.1 > now {
                first_valid = i as i32;
                break;
            }
        }

        // no valid files, remove all of them
        if first_valid == -1 {
            println!("All holders ({}) expired.", holders.holders.len());
            //market_data.files.remove(&file_hash);
            holders.holders.clear();
        } else {
            if first_valid > 0 {
                println!("Found {} expired files", first_valid);
                // remove expired times
                holders.holders.drain(0..first_valid as usize);
            }

            for holder in &holders.holders {
                users.push(holder.clone());
            }
        }
        if let Err(err) = self
            .dht_client
            .set_holders(file_hash.as_str(), holders.clone())
            .await
        {
            eprintln!("Error: {:?}", err);
        }

        //market_data.print_holders_map();

        Ok(Some(HoldersResponse {
            file_info: Some(holders.file_info),
            holders: users.into_iter().map(|(user, _exp)| user).collect(),
        }))
    }

    pub async fn stop(self) -> Result<()> {
        let res = self.dht_client.stop().await;
        self.dht_handle.abort();
        println!("Stopped market client: {res:?}");
        res
    }

    async fn insert_and_validate(&self, user: User, file_info: FileInfo, expiration: u64) {
        let Ok(file_holders) = self
            .dht_client
            .get::<FileHolders>(&file_info.file_hash)
            .await
        else {
            eprintln!("Failed to fetch file requests from Kad");
            return;
        };
        let mut file_holders = file_holders.unwrap_or(FileHolders {
            file_info: file_info.clone(),
            holders: vec![],
        });
        let current_time = get_current_time();
        file_holders
            .holders
            .retain(|(holder, exp)| *exp >= current_time && holder.id != user.id);
        file_holders.holders.push((user, expiration));
        match self
            .dht_client
            .set_holders(file_info.hash().as_str(), file_holders)
            .await
        {
            Ok(_) => {}
            Err(_) => eprintln!("Failed to update file requests in Kad"),
        }
    }
}
