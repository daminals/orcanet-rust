use rand::Rng;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

type AsyncJob = Arc<Mutex<Job>>;

#[derive(Clone)]
pub struct Jobs {
    pub jobs: Arc<RwLock<HashMap<String, AsyncJob>>>,
    pub history: Arc<RwLock<HashMap<String, HistoryEntry>>>,
}

#[derive(Clone)]
pub struct Job {
    file_hash: String,
    file_name: String,
    file_size: u64,
    time_queued: u64, // unix time in seconds
    status: String,
    accumulated_cost: u64,
    projected_cost: u64,
    eta: u64, // seconds
    peerId: String,
}

#[derive(Serialize)]
pub struct JobListItem {
    jobID: String,
    fileName: String,
    fileSize: u64,
    eta: u64,
    timeQueued: u64,
    status: String,
}

#[derive(Clone)]
pub struct HistoryEntry {
    fileName: String,
    timeCompleted: u64, // unix time in seconds
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_job(
        &self,
        file_hash: String,
        file_size: u64,
        filename: String,
        peer_id: String,
    ) -> String {
        let job_id = rand::thread_rng().gen::<u64>().to_string();

        // Get a write lock on the jobs map
        let mut jobs = self.jobs.write().await;

        // Add the job to the map
        let job = Job {
            file_hash: file_hash.clone(),
            file_name: filename.clone(),
            file_size: 0,
            time_queued: 0,
            status: "queued".to_string(),
            accumulated_cost: 0,
            projected_cost: 0,
            eta: 0,
            peerId: peer_id.clone(),
        };
        let async_job = Arc::new(Mutex::new(job));
        jobs.insert(job_id.clone(), async_job.clone());

        // Get a write lock on the job history map
        let mut history = self.history.write().await;

        // Add the job to the history map
        history.insert(
            job_id.clone(),
            HistoryEntry {
                fileName: filename,
                timeCompleted: 0,
            },
        );

        job_id
    }

    pub async fn get_job(&self, job_id: &str) -> Option<AsyncJob> {
        // Get a read lock on the jobs map
        let jobs = self.jobs.read().await;

        // Get the job
        jobs.get(job_id).cloned()
    }

    pub async fn get_jobs_list(&self) -> Vec<JobListItem> {
        // Get a read lock on the jobs map
        let jobs = self.jobs.read().await;

        let mut jobs_list = vec![];

        // might not be ideal, but idk how to do async with map
        for job in jobs.values() {
            let job = job.lock().await;

            let job_item = JobListItem {
                jobID: job.file_hash.clone(),
                fileName: job.file_name.clone(),
                fileSize: job.file_size, // TODO
                eta: job.eta,
                timeQueued: job.time_queued,
                status: job.status.clone(),
            };
            jobs_list.push(job_item);
        }

        jobs_list
    }

    pub async fn get_job_history(&self) -> Vec<HistoryEntry> {
        // Get a read lock on the job history
        let history = self.history.read().await;

        history.values().cloned().collect()
    }

    pub async fn remove_job_from_history(&self, job_id: &str) {
        // Get a write lock on the job history
        let mut history = self.history.write().await;

        history.remove(job_id);
    }

    pub async fn clear_job_history(&self) {
        // Get a write lock on the job history
        let mut history = self.history.write().await;

        history.clear();
    }
}
