use rand::Rng;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use tokio::sync::{Mutex, RwLock};

type AsyncJob = Arc<Mutex<Job>>;

#[derive(Clone)]
pub struct Jobs {
    pub jobs: Arc<RwLock<HashMap<String, AsyncJob>>>,
    pub history: Arc<RwLock<HashMap<String, HistoryEntry>>>,
}

#[derive(Clone)]
pub struct Job {
    job_id: String,
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

pub fn current_time_secs() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
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
        file_name: String,
        price: i64,
        peer_id: String,
    ) -> String {
        // generate a random job id
        let job_id = rand::thread_rng().gen::<u64>().to_string();

        // Get a write lock on the jobs map
        let mut jobs = self.jobs.write().await;

        // Add the job to the map
        let job = Job {
            job_id: job_id.clone(),
            file_hash: file_hash.clone(),
            file_name: file_name.clone(),
            file_size,
            time_queued: current_time_secs(),
            status: "active".to_string(),
            accumulated_cost: 0,
            projected_cost: file_size * price as u64,
            eta: 0, // TODO, have correct eta
            peerId: peer_id.clone(),
        };
        let async_job = Arc::new(Mutex::new(job));
        jobs.insert(job_id.clone(), async_job.clone());

        job_id
    }

    pub async fn get_job(&self, job_id: &str) -> Option<AsyncJob> {
        let jobs = self.jobs.read().await;

        jobs.get(job_id).cloned()
    }

    pub async fn get_jobs_list(&self) -> Vec<JobListItem> {
        let jobs = self.jobs.read().await;

        let mut jobs_list = vec![];

        // might not be ideal, but idk how to do async with map
        for job in jobs.values() {
            let job = job.lock().await;

            let job_item = JobListItem {
                jobID: job.job_id.clone(),
                fileName: job.file_name.clone(),
                fileSize: job.file_size,
                eta: job.eta,
                timeQueued: job.time_queued,
                status: job.status.clone(),
            };
            jobs_list.push(job_item);
        }

        jobs_list
    }


    pub async fn finish_job(&self, job_id: &str) {
        let mut jobs = self.jobs.write().await;
        let mut job = jobs.get_mut(job_id).unwrap().lock().await;

        // mark the job as completed
        job.status = "completed".to_string();

        // add the completed job to history
        let mut history = self.history.write().await;
        let history_entry = HistoryEntry {
            fileName: job.file_name.clone(),
            timeCompleted: 0,
        };
        history.insert(job_id.to_string(), history_entry);
    }

    pub async fn get_job_history(&self) -> Vec<HistoryEntry> {
        let history = self.history.read().await;

        history.values().cloned().collect()
    }

    pub async fn remove_job_from_history(&self, job_id: &str) {
        let mut history = self.history.write().await;

        history.remove(job_id);
    }

    pub async fn clear_job_history(&self) {
        let mut history = self.history.write().await;

        history.clear();
    }
}
