use core::fmt;
use rand::Rng;
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
};

use crate::{
    consumer::{encode, get_file_chunk, http::GetFileResponse},
    store::Configurations,
};

type AsyncJob = Arc<Mutex<Job>>;

#[derive(Clone)]
pub struct Jobs {
    pub jobs: Arc<RwLock<HashMap<String, AsyncJob>>>,
    pub history: Arc<RwLock<HashMap<String, HistoryEntry>>>,
}

#[derive(Debug)]
pub struct Job {
    job_id: String,
    file_hash: String,
    file_name: String,
    file_size: u64,
    time_queued: u64, // unix time in seconds
    status: JobStatus,
    accumulated_cost: u64,
    projected_cost: u64,
    eta: u64, // seconds
    pub peer_id: String,
}

#[derive(Debug)]
pub enum JobStatus {
    Active(JoinHandle<()>),
    Stop,        // wait to stop
    Paused(u64), // chunk num of next
    Completed,
    Failed,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Active(_) => write!(f, "active"),
            JobStatus::Paused(_) => write!(f, "paused"),
            JobStatus::Stop => write!(f, "stopping"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
        }
    }
}

pub async fn start(job: AsyncJob, token: String) {
    let mut lock = job.lock().await;
    println!("Starting job with token {token}");
    if let JobStatus::Paused(next_chunk) = lock.status {
        let job = job.clone();
        dbg!(&lock);
        let producer = lock.peer_id.clone();
        let producer_user = match encode::decode_user(producer.clone()) {
            Ok(user) => user,
            Err(e) => {
                eprintln!("Failed to decode producer: {}", e);
                lock.status = JobStatus::Failed;
                return;
            }
        };
        lock.status = JobStatus::Active(tokio::spawn(async move {
            let mut lock = job.lock().await;
            let file_hash = lock.file_hash.clone();
            let mut chunk_num = next_chunk;
            let mut return_token = String::from(token);
            loop {
                drop(lock);
                match get_file_chunk(
                    producer_user.clone(),
                    file_hash.clone(),
                    return_token.clone(),
                    chunk_num,
                )
                .await
                {
                    Ok(response) => {
                        match response {
                            GetFileResponse::Token(new_token) => {
                                return_token = new_token;
                            }
                            GetFileResponse::Done => {
                                println!("Consumer: File downloaded successfully");
                                lock = job.lock().await;
                                lock.status = JobStatus::Completed;
                                return;
                            }
                        }
                        chunk_num += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to download chunk {}: {}", chunk_num, e);
                        lock = job.lock().await;
                        lock.status = JobStatus::Completed;
                        return;
                    }
                }
                lock = job.lock().await;
                if let JobStatus::Stop = lock.status {
                    lock.status = JobStatus::Paused(chunk_num);
                }
            }
        }));
    }
}

#[derive(Serialize)]
pub struct JobListItem {
    job_id: String,
    file_name: String,
    file_size: u64,
    eta: u64,
    time_queued: u64,
    status: String,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct JobInfo {
    fileHash: String,
    fileName: String,
    fileSize: u64,
    accumulatedMemory: u64,
    accumulatedCost: u64,
    projectedCost: u64,
    eta: u64,
}
#[derive(Clone, Serialize)]
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
            status: JobStatus::Paused(0),
            accumulated_cost: 0,
            projected_cost: file_size * price as u64,
            eta: 0, // TODO, have correct eta
            peer_id: peer_id.clone(),
        };
        let async_job = Arc::new(Mutex::new(job));
        jobs.insert(job_id.clone(), async_job.clone());

        job_id
    }

    pub async fn get_job(&self, job_id: &str) -> Option<AsyncJob> {
        let jobs = self.jobs.read().await;

        jobs.get(job_id).cloned()
    }
    pub async fn get_job_info(&self, job_id: &str) -> Option<JobInfo> {
        let jobs = self.jobs.read().await;

        let job = match jobs.get(job_id) {
            Some(job) => job.clone(),
            None => return None,
        };

        let job = job.lock().await;

        let job_info = JobInfo {
            fileHash: job.file_hash.clone(),
            fileName: job.file_name.clone(),
            fileSize: job.file_size,
            accumulatedMemory: 0, //TODO
            accumulatedCost: job.accumulated_cost,
            projectedCost: job.projected_cost,
            eta: job.eta,
        };

        Some(job_info)
    }

    pub async fn get_jobs_list(&self) -> Vec<JobListItem> {
        let jobs = self.jobs.read().await;

        let mut jobs_list = vec![];

        // might not be ideal, but idk how to do async with map
        for job in jobs.values() {
            let job = job.lock().await;

            let job_item = JobListItem {
                job_id: job.job_id.clone(),
                file_name: job.file_name.clone(),
                file_size: job.file_size,
                eta: job.eta,
                time_queued: job.time_queued,
                status: job.status.to_string(),
            };
            jobs_list.push(job_item);
        }

        jobs_list
    }

    pub async fn finish_job(&self, job_id: &str) -> bool {
        let mut jobs = self.jobs.write().await;

        // get the job, returning false if it doesn't exist
        let job = match jobs.get_mut(job_id) {
            Some(job) => job.clone(),
            None => return false,
        };

        let mut job = job.lock().await;

        if let JobStatus::Completed = job.status {
            return false;
        }

        // mark the job as completed
        job.status = JobStatus::Completed;

        // add the completed job to history
        let mut history = self.history.write().await;
        let history_entry = HistoryEntry {
            fileName: job.file_name.clone(),
            timeCompleted: 0,
        };
        history.insert(job_id.to_string(), history_entry);

        true
    }

    pub async fn get_job_history(&self) -> Vec<HistoryEntry> {
        let history = self.history.read().await;

        history.values().cloned().collect()
    }

    pub async fn remove_job_from_history(&self, job_id: &str) -> bool {
        let mut history = self.history.write().await;

        history.remove(job_id).is_some()
    }

    pub async fn clear_job_history(&self) {
        let mut history = self.history.write().await;

        history.clear();
    }
}
