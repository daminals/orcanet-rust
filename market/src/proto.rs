use ::proto::market::{FileInfo, User};
use serde::{Deserialize, Serialize};

// store holders with u64 timestamp for expiration
// when sending, drop timestamp
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileHolders {
    pub file_info: FileInfo,
    // (User, expiration)
    pub holders: Vec<(User, u64)>,
}

//impl TryFrom<HoldersResponse> for FileHolders {
//    type Error = ();
//    // unwrap holders
//    fn try_from(h: HoldersResponse) -> Result<Self, Self::Error> {
//        Ok(Self {
//            file_info: h.file_info.ok_or(())?,
//            holders: h.holders,
//        })
//    }
//}
