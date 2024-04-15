use rand::Rng;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

type AsyncConsumer = Arc<Mutex<Consumer>>;

pub struct Consumers {
    consumers: RwLock<HashMap<String, AsyncConsumer>>,
}

#[derive(Clone)]
pub struct ConsumerRequest {
    pub chunks_sent: u64,
    pub access_token: String,
}

#[derive(Clone)]
pub struct Consumer {
    pub wallet_address: String,
    pub requests: HashMap<String, ConsumerRequest>,
}

impl Consumers {
    pub fn new() -> Self {
        Consumers {
            consumers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_consumer(&self, consumer_ip: String, consumer: Consumer) -> AsyncConsumer {
        // Get a write lock on the consumers map
        let mut consumers = self.consumers.write().await;

        // Add the consumer to the map
        let async_consumer = Arc::new(Mutex::new(consumer));
        consumers.insert(consumer_ip, async_consumer.clone());
        async_consumer
    }

    pub async fn get_consumer(&self, consumer_address: &str) -> Option<AsyncConsumer> {
        // Get a read lock on the consumers map
        let consumers = self.consumers.read().await;

        // Get the consumer
        consumers.get(consumer_address).cloned()
    }
}

pub fn generate_access_token() -> String {
    // Generate a completely random access token
    let mut rng = rand::thread_rng();
    let token: String = (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();

    token
}
