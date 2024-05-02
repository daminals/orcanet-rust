use core::fmt;
use proto::market::FileInfoHash;
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

use peernode::consumer::{
    encode::{self, EncodedUser},
    get_file_chunk,
    http::GetFileResponse,
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
    file_info_hash: FileInfoHash,
    file_name: String,
    file_size: u64,
    time_queued: u64, // unix time in seconds
    status: JobStatus,
    accumulated_cost: u64,
    projected_cost: u64,
    eta: u64, // seconds
    pub peer_id: String,
    pub encoded_producer: EncodedUser,
}

impl Job {
    pub fn pause(&mut self) -> bool {
        if let JobStatus::Active(_) = self.status {
            self.status = JobStatus::Stop;

            return true;
        }

        false
    }
    pub fn terminate(&mut self) -> bool {
        match &mut self.status {
            JobStatus::Active(handle) => {
                handle.abort();
                self.status = JobStatus::Failed;
                return true;
            }
            JobStatus::Completed => {
                return false;
            }
            _ => {
                self.status = JobStatus::Failed;
                return true;
            }
        }
    }
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

pub async fn start(job: AsyncJob, jobs: Arc<Mutex<Jobs>>, token: String) -> bool {
    let mut lock = job.lock().await;

    match lock.status {
        JobStatus::Completed | JobStatus::Failed => {
            return false;
        }
        _ => {}
    }

    println!("Starting job with token {token}");
    if let JobStatus::Paused(next_chunk) = lock.status {
        let job = job.clone();
        dbg!(&lock);
        let producer_user = match encode::try_decode_user(lock.encoded_producer.as_str()) {
            Ok(user) => user,
            Err(e) => {
                eprintln!("Failed to decode producer: {e}");
                lock.status = JobStatus::Failed;
                return false;
            }
        };
        lock.status = JobStatus::Active(tokio::spawn(async move {
            let mut lock = job.lock().await;
            let file_info_hash = lock.file_info_hash.clone();
            let mut chunk_num = next_chunk;
            let mut return_token = token;
            loop {
                drop(lock);
                match get_file_chunk(
                    producer_user.clone(),
                    file_info_hash.clone(),
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

                                let history_entry = HistoryEntry {
                                    fileName: lock.file_name.clone(),
                                    timeCompleted: current_time_secs(),
                                };

                                jobs.lock()
                                    .await
                                    .add_job_to_history(&lock.job_id, history_entry)
                                    .await;

                                return;
                            }
                        }
                        chunk_num += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to download chunk {chunk_num}: {e}");
                        lock = job.lock().await;
                        lock.status = JobStatus::Failed;
                        return;
                    }
                }
                lock = job.lock().await;
                if let JobStatus::Stop = lock.status {
                    lock.status = JobStatus::Paused(chunk_num);
                    return;
                }
            }
        }));
    }

    true
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct JobListItem {
    jobID: String,
    fileName: String,
    fileSize: u64,
    eta: u64,
    timeQueued: u64,
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
#[allow(non_snake_case)]
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
        file_info_hash: FileInfoHash,
        file_size: u64,
        file_name: String,
        price: i64,
        peer_id: String,
        encoded_producer: EncodedUser,
    ) -> String {
        // generate a random job id
        let job_id = rand::thread_rng().gen::<u64>().to_string();

        // Get a write lock on the jobs map
        let mut jobs = self.jobs.write().await;

        // Add the job to the map
        let job = Job {
            job_id: job_id.clone(),
            file_info_hash: file_info_hash.clone(),
            file_name: file_name.clone(),
            file_size,
            time_queued: current_time_secs(),
            status: JobStatus::Paused(0),
            accumulated_cost: 0,
            projected_cost: file_size * price as u64,
            eta: 0, // TODO, have correct eta
            peer_id: peer_id.clone(),
            encoded_producer,
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
            fileHash: job.file_info_hash.as_str().to_owned(),
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
                jobID: job.job_id.clone(),
                fileName: job.file_name.clone(),
                fileSize: job.file_size,
                eta: job.eta,
                timeQueued: job.time_queued,
                status: job.status.to_string(),
            };
            jobs_list.push(job_item);
        }

        jobs_list
    }

    pub async fn add_job_to_history(&self, job_id: &str, entry: HistoryEntry) -> bool {
        // add the completed job to history
        let mut history = self.history.write().await;

        history.insert(job_id.to_owned(), entry);

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
