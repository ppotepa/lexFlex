use std::fs;
use std::io::Read;
use std::path::Path;

use super::{SessionStoreError, MAX_SESSION_RECORD_BYTES};

pub(super) fn read_bounded(path: &Path) -> Result<Vec<u8>, SessionStoreError> {
    let file = fs::File::open(path).map_err(|error| SessionStoreError::Io {
        path: path.to_owned(),
        kind: error.kind(),
    })?;
    let mut bytes = Vec::new();
    file.take((MAX_SESSION_RECORD_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| SessionStoreError::Io {
            path: path.to_owned(),
            kind: error.kind(),
        })?;
    if bytes.len() > MAX_SESSION_RECORD_BYTES {
        return Err(SessionStoreError::ResourceLimit);
    }
    Ok(bytes)
}
