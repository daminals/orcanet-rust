use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use glob::glob;
use sha2::Digest;
use sha2::Sha256;
use std::fs::File;
use std::io;
use tokio::io::SeekFrom;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::RwLock;

pub struct FileMap {
    files: RwLock<HashMap<String, PathBuf>>,
}

pub type AsyncFileMap = Arc<FileMap>;

impl FileMap {
    pub fn new() -> Self {
        FileMap {
            files: RwLock::new(HashMap::new()),
        }
    }

    // Add all the files in a Unix-style glob to the map
    pub async fn add_all(&self, file_path: &str) -> Result<()> {
        // Get a write lock on the files map
        let mut files = self.files.write().await;
        for entry in glob(file_path)? {
            let path = entry?;
            let file = File::open(path.clone());
            match file {
                Ok(mut file) => {
                    let hash = self.hash_file(&mut file)?;
                    files.insert(hash, path);
                }
                Err(_) => {
                    eprintln!("Failed to open file {:?}", path);
                }
            }
        }

        Ok(())
    }

    // Get a file path by its hash
    pub async fn get_file_path(&self, hash: &str) -> Option<PathBuf> {
        // Get a read lock on the files map
        let files = self.files.read().await;

        let path = files.get(hash)?;
        Some(path.clone())
    }

    // Get a vector of all the hashes in the map
    pub async fn get_hashes(&self) -> Vec<String> {
        // Get a read lock on the files map
        let files = self.files.read().await;

        files.keys().cloned().collect()
    }

    // Get the hash of a file
    fn hash_file(&self, file: &mut File) -> Result<String> {
        let mut sha256 = Sha256::new();
        io::copy(file, &mut sha256)?;
        let hash = sha256.finalize();
        let hash = format!("{:x}", hash);
        Ok(hash)
    }
}

pub struct FileAccessType {
    file_path: String,
}

impl FileAccessType {
    // chunk size = 4mb
    const CHUNK_SIZE: u64 = 4 * 1024 * 1024;

    pub fn new(file: &str) -> Result<Self> {
        Ok(FileAccessType {
            file_path: file.to_string(),
        })
    }

    pub async fn get_chunk(&self, desired_chunk: u64) -> Result<Option<Vec<u8>>> {
        // open the file
        let mut file = tokio::fs::File::open(&self.file_path).await?;

        // get total chunk number (file size / chunk size)
        let metadata = file.metadata().await?;
        let total_chunks = metadata.len() / Self::CHUNK_SIZE;

        // create a buffer to hold the file data
        let mut buffer = vec![0; Self::CHUNK_SIZE as usize];

        // check if the desired chunk is within the file size
        if desired_chunk > total_chunks {
            return Ok(None);
        }

        // seek to the desired chunk
        file.seek(SeekFrom::Start(desired_chunk * Self::CHUNK_SIZE))
            .await?;

        // read the chunk into the buffer
        let res = file.read_exact(&mut buffer).await;
        match res {
            Ok(_) => Ok(Some(buffer)),
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    // read until the end of the file
                    file.seek(SeekFrom::Start(desired_chunk * Self::CHUNK_SIZE))
                        .await?;
                    let mut buffer = vec![];
                    file.read_to_end(&mut buffer).await?;
                    Ok(Some(buffer))
                } else {
                    Err(e.into())
                }
            }
        }
    }
}
