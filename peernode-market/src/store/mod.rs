use crate::market::Market;
use crate::producer;
use anyhow::Result;
use config::{Config, File, FileFormat};
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive()]
pub struct Configurations {
    // this is the struct that will be used to store the configurations
    props: Properties,
    http_client: Option<tokio::task::JoinHandle<()>>,
    market_client: Option<Market>,
}

#[derive(Serialize, Deserialize)]
pub struct Properties {
    // must be a separate serializable struct so can read from config.json file
    name: String,
    files: HashMap<String, PathBuf>,
    prices: HashMap<String, i64>,
    chunk_metadatas: HashMap<String, Vec<(String, u64)>>,
    tokens: HashMap<String, String>,
    port: String,
    // market config
    bootstrap_peers: Vec<Multiaddr>,
    listen_addr: Option<Multiaddr>,
    private_key: Option<String>,
    // wallet: String, // not sure about implementation details, will revisit later
}

// TODO: Put prices and path attached to the same hash in config file, and then construct the hashmaps from that
impl Configurations {
    pub async fn new() -> Self {
        let config = Config::builder()
            .add_source(File::new("config", FileFormat::Json))
            .build();
        let props = match config {
            Ok(config) => {
                // lets try to deserialize the configuration
                let config_data = config.try_deserialize::<Properties>();
                match config_data {
                    Ok(config_data) => config_data,
                    Err(_) => {
                        return Self::default().await;
                    }
                }
            }
            Err(_) => {
                return Self::default().await;
            }
        };
        Configurations {
            props,
            http_client: None,
            market_client: None,
        }
    }

    pub async fn default() -> Self {
        // this is the default configuration
        let default = Configurations {
            props: Properties {
                name: "default".to_string(),
                files: HashMap::new(),
                prices: HashMap::new(),
                chunk_metadatas: HashMap::new(),
                tokens: HashMap::new(),
                port: "8080".to_string(),
                bootstrap_peers: vec![],
                //listen_addr: Some(Multiaddr::from_str("/ip4/0.0.0.0/tcp/6881").unwrap()),
                listen_addr: None,
                //private_key: Some("private.pk8".to_owned()),
                private_key: None,
            },
            http_client: None,
            market_client: None,
        };
        default.write();
        default
    }

    // write to config.json
    pub fn write(&self) {
        // Serialize it to a JSON string.
        match serde_json::to_string(&self.props) {
            Ok(default_config_json) => {
                // Write the string to the file.
                match std::fs::write("config.json", default_config_json) {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("Failed to write to file");
                    }
                }
            }
            Err(_) => {
                eprintln!("Failed to serialize configuration");
            }
        }
    }

    pub fn get_hash(&self, file_path: String) -> Result<String> {
        // open the file
        let mut file = std::fs::File::open(file_path)?;
        // hash the file
        let hash = producer::files::hash_file(&mut file)?;
        Ok(hash)
    }

    pub fn get_chunk_metadata(&self, file_path: String) -> Result<Vec<(String, u64)>> {
        // open the file
        let mut file = std::fs::File::open(file_path)?;
        // chunk metadata the file
        let chunk_metadata = producer::files::generate_chunk_metadata(&mut file)?;
        Ok(chunk_metadata)
    }

    pub fn get_files(&self) -> HashMap<String, PathBuf> {
        self.props.files.clone()
    }

    pub fn get_prices(&self) -> HashMap<String, i64> {
        self.props.prices.clone()
    }

    pub fn get_chunk_metadatas(&self) -> HashMap<String, Vec<(String, u64)>> {
        self.props.chunk_metadatas.clone()
    }

    pub fn get_port(&self) -> String {
        self.props.port.clone()
    }

    pub fn get_bootstrap_peers(&self) -> Vec<Multiaddr> {
        self.props.bootstrap_peers.clone()
    }

    pub fn get_listen_address(&self) -> Option<Multiaddr> {
        self.props.listen_addr.clone()
    }

    pub fn get_private_key(&self) -> Option<String> {
        self.props.private_key.clone()
    }

    pub fn get_token(&mut self, producer_id: String) -> String {
        match self.props.tokens.get(&producer_id).cloned() {
            Some(token) => token,
            None => {
                let token = "token".to_string();
                self.set_token(producer_id, token.clone());
                self.write();
                token
            }
        }
    }

    pub fn set_token(&mut self, producer_id: String, token: String) {
        self.props.tokens.insert(producer_id, token);
        self.write();
    }

    pub fn set_port(&mut self, port: String) {
        self.props.port = port;
        self.write();
    }

    pub fn set_bootstrap_peers(&mut self, bootstrap_peers: Vec<Multiaddr>) {
        self.props.bootstrap_peers = bootstrap_peers;
        self.write();
    }

    pub fn set_listen_address(&mut self, listen_address: Option<Multiaddr>) {
        self.props.listen_addr = listen_address;
    }

    pub fn set_private_key(&mut self, private_key: Option<String>) {
        self.props.private_key = private_key
    }

    // add every file in the directory to the list
    pub fn add_dir(&mut self, file_path: String, price: i64) -> Result<()> {
        // assume that the file_path is a directory
        for entry in fs::read_dir(file_path)? {
            let path = entry?.path();
            // convert the path to a string
            let path_string = match path.to_str() {
                Some(path) => path,
                None => {
                    panic!("Failed to convert path to string");
                }
            };
            // check if this is a file or a directory
            if path.is_dir() {
                self.add_dir(path_string.to_owned(), price)?;
            }
            if path.is_file() {
                self.add_file(path_string.to_owned(), price)
            }
        }
        Ok(())
    }

    // add a single file to the list
    pub fn add_file(&mut self, file: String, price: i64) {
        // hash the file
        let hash = match self.get_hash(file.clone()) {
            Ok(hash) => hash,
            Err(_) => {
                panic!("Failed to hash file");
            }
        };

        // get the file's chunk metadata
        let chunk_metadata = match self.get_chunk_metadata(file.clone()) {
            Ok(chunk_metadata) => chunk_metadata,
            Err(_) => {
                panic!("Failed to get chunk metadata");
            }
        };    

        self.props.files.insert(hash.clone(), PathBuf::from(file));
        self.props.prices.insert(hash.clone(), price);
        self.props.chunk_metadatas.insert(hash, chunk_metadata);
        // self.write();
    }

    // cli command to add a file/dir to the list
    pub fn add_file_path(&mut self, file: String, price: i64) {
        // check if this is a file or a directory
        match std::fs::metadata(&file) {
            Ok(metadata) => {
                if metadata.is_file() {
                    self.add_file(file.clone(), price);
                }
                if metadata.is_dir() {
                    match self.add_dir(file.clone(), price) {
                        Ok(_) => {}
                        Err(_) => {
                            eprintln!("Failed to add directory {}", file);
                        }
                    };
                }
            }
            Err(_) => {
                eprintln!("Failed to open file {:?}", file);
            }
        }
        self.write();
    }

    pub fn remove_file(&mut self, file_path: String) {
        // get the hash of the file
        let hash = match self.get_hash(file_path.clone()) {
            Ok(hash) => hash,
            Err(_) => {
                panic!("Failed to hash file");
            }
        };

        // if file is not in the list, panic
        if !self.props.files.contains_key(&hash) || !self.props.prices.contains_key(&hash) {
            panic!("File [{}] not found", file_path);
        }
        self.props.files.remove(&hash);
        self.props.prices.remove(&hash);
        self.write();
    }

    pub fn set_http_client(&mut self, http_client: tokio::task::JoinHandle<()>) {
        self.http_client = Some(http_client);
    }

    pub fn is_http_running(&self) -> bool {
        self.http_client.is_some()
    }

    pub async fn start_http_client(&mut self, port: String) {
        // stop the current http client
        if let Some(http_client) = self.http_client.take() {
            match producer::stop_server(http_client).await {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("Failed to stop HTTP server");
                }
            }
        }

        // Set the port
        self.set_port(port.clone());

        let join = // must run in separate thread so does not block cli inputs
            producer::start_server(self.props.files.clone(), self.props.prices.clone(), port).await;
        self.set_http_client(join);
    }

    pub async fn stop_http_client(&mut self) {
        if let Some(http_client) = self.http_client.take() {
            match producer::stop_server(http_client).await {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("Failed to stop HTTP server");
                }
            }
        }
    }

    pub async fn get_market_client(&mut self) -> Result<&mut Market> {
        if self.market_client.is_none() {
            let market_client = Market::new(
                &self.get_bootstrap_peers(),
                self.get_private_key(),
                self.get_listen_address(),
            )
            .await?;
            self.market_client = Some(market_client);
        }
        let market_client = self.market_client.as_mut().unwrap(); // safe to unwrap because we just set it
        Ok(market_client)
    }

    pub async fn set_market_client(
        &mut self,
        bootstrap_peers: Vec<Multiaddr>,
        private_key: Option<String>,
        listen_address: Option<Multiaddr>,
    ) -> Result<&mut Market> {
        self.set_bootstrap_peers(bootstrap_peers);
        self.set_private_key(private_key);
        self.set_listen_address(listen_address);
        if let Some(old_client) = self.market_client.take() {
            old_client.stop().await?;
        }
        let market_client = Market::new(
            &self.get_bootstrap_peers(),
            self.get_private_key(),
            self.get_listen_address(),
        )
        .await?;
        self.market_client = Some(market_client);
        Ok(self.market_client.as_mut().unwrap()) // safe to unwrap because we just set it
    }
}
