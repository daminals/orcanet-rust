use rand::Rng;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

type AsyncJob = Arc<Mutex<Job>>;

pub struct Jobs {
    jobs: RwLock<HashMap<String, AsyncJob>>,
    history: RwLock<HashMap<String, HistoryEntry>>,
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

#[derive(Clone)]
pub struct HistoryEntry {
    fileName: String,
    timeCompleted: u64, // unix time in seconds
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            jobs: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_job(&self, peerId: String, fileName: string, job: Job) -> String {
        let jobId = rand::thread_rng().gen::<u64>().to_string();

        // Get a write lock on the jobs map
        let mut jobs = self.jobs.write().await;

        // Add the job to the map
        let async_job = Arc::new(Mutex::new(job));
        jobs.insert(jobId, async_job.clone());

        // Get a write lock on the job history map
        let mut history = self.history.write().await;

        // Add the job to the history map
        history.insert(jobId, HistoryEntry {
            fileName,
            timeCompleted: -1,
        });

        job_id
    }

    pub async fn get_job(&self, jobId: &str) -> Option<AsyncConsumer> {
        // Get a read lock on the jobs map
        let jobs = self.jobs.read().await;

        // Get the job
        jobs.get(jobId).cloned()
    }
    
    pub async fn get_job_history(&self) -> Vec<Job> {
        // Get a read lock on the job history
        let history = self.history.read().await;

        history.values().cloned().collect()
    }

    pub async fn remove_job_from_history(&self, jobId: &str) {
        // Get a write lock on the job history
        let mut history = self.history.write().await;

        history.remove(jobId);
    }

    pub async fn clear_job_history(&self) {
        // Get a write lock on the job history
        let mut history = self.history.write().await;

        history.clear();
    }
}
