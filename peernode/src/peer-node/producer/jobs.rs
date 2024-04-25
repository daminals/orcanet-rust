use rand::Rng;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

type AsyncJob = Arc<Mutex<Job>>;

pub struct Jobs {
    jobs: RwLock<HashMap<String, AsyncJob>>,
    history: RwLock<Vec<Job>>,
}

#[derive(Clone)]
pub struct Job {
    fileHash: String,
    timeQueued: u64, // unix time in seconds
    status: String,
    accumulatedCost: u64,
    projectedCost: u64,
    eta: u64, // seconds
    peerId: String,
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            jobs: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_job(&self, peerId: String, job: Job) -> AsyncJob {
        // Get a write lock on the jobs map
        let mut jobs = self.jobs.write().await;

        // Add the consumer to the map
        let async_job = Arc::new(Mutex::new(job));
        jobs.insert(peerId, async_job.clone());

        // Get a write lock on the job history
        let mut history = self.history.write().await;

        // Add the job to the history
        history.push(async_job.lock().await.clone());

        async_job
    }

    pub async fn get_job(&self, peerId: &str) -> Option<AsyncConsumer> {
        // Get a read lock on the jobs map
        let jobs = self.jobs.read().await;

        // Get the job
        jobs.get(peerId).cloned()
    }
    
    pub async fn get_job_history(&self) -> Vec<Job> {
        // Get a read lock on the job history
        let history = self.history.read().await;

        history.clone()
    }

    pub async fn remove_job_from_history(&self, peerId: &str) {
        // Get a write lock on the job history
        let mut history = self.history.write().await;

        // Remove the job from the history
        history.retain(|job| job.peerId != peerId);
    }

    pub async fn clear_job_history(&self) {
        // Get a write lock on the job history
        let mut history = self.history.write().await;

        history.clear();
    }
}
