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
            history: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_job(&self, filename: String, job: Job) -> String {
        let job_id = rand::thread_rng().gen::<u64>().to_string();

        // Get a write lock on the jobs map
        let mut jobs = self.jobs.write().await;

        // Add the job to the map
        let async_job = Arc::new(Mutex::new(job));
        jobs.insert(job_id.clone(), async_job.clone());

        // Get a write lock on the job history map
        let mut history = self.history.write().await;

        // Add the job to the history map
        history.insert(job_id.clone(), HistoryEntry {
            fileName: filename,
            timeCompleted: 0,
        });

        job_id
    }

    pub async fn get_job(&self, job_id: &str) -> Option<AsyncJob> {
        // Get a read lock on the jobs map
        let jobs = self.jobs.read().await;

        // Get the job
        jobs.get(job_id).cloned()
    }
    
    pub async fn get_job_history(&self) -> Vec<HistoryEntry> {
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
