pub mod market {
    tonic::include_proto!("market"); // The string specified here must match the proto package name

    use core::fmt;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};
    use std::hash::{Hash, Hasher};

    impl Hash for FileInfo {
        fn hash<H: Hasher>(&self, state: &mut H) {
            let mut input = self.file_hash.clone();
            for chunk_hash in &self.chunk_hashes {
                input += chunk_hash;
            }
            input += self.file_size.to_string().as_str();
            input += self.file_name.as_str();
            input.hash(state);
        }
    }

    impl FileInfo {
        pub fn get_hash(&self) -> FileInfoHash {
            let mut sha256 = Sha256::new();
            let mut input = self.file_hash.clone();
            for chunk_hash in &self.chunk_hashes {
                input += chunk_hash;
            }
            input += self.file_size.to_string().as_str();
            input += self.file_name.as_str();
            sha256.update(input);
            FileInfoHash(format!("{:x}", sha256.finalize()))
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[repr(transparent)]
    pub struct FileInfoHash(String);

    impl FileInfoHash {
        #[inline(always)]
        pub const fn new(s: String) -> Self {
            Self(s)
        }

        #[inline(always)]
        pub fn into_bytes(self) -> Vec<u8> {
            self.into()
        }

        #[inline(always)]
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl From<String> for FileInfoHash {
        fn from(s: String) -> Self {
            Self(s)
        }
    }

    impl From<FileInfoHash> for Vec<u8> {
        fn from(value: FileInfoHash) -> Self {
            value.0.into_bytes()
        }
    }

    impl fmt::Display for FileInfoHash {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}
