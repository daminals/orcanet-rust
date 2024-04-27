pub mod market {
use sha2::{Digest, Sha256};
    tonic::include_proto!("market"); // The string specified here must match the proto package name

    impl FileInfo {
        pub fn hash(self: &FileInfo) -> String {
            let mut sha256 = Sha256::new();
            let mut input = self.file_hash.clone();
            for chunk_hash in &self.chunk_hashes {
                input += chunk_hash;
            }
            input += self.file_size.to_string().as_str();
            input += self.file_name.as_str();
            sha256.update(input);
            format!("{:x}", sha256.finalize())
        }
    }

}
