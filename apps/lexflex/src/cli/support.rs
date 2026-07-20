use crate::cli::input_error::CliLocalError;
use serde::de::DeserializeOwned;
use std::io::Read;
use std::path::Path;

const MAX_RON_BYTES: usize = 32 * 1024 * 1024;

pub fn read_ron<T>(path: &Path) -> Result<T, CliLocalError>
where
    T: DeserializeOwned,
{
    let file = std::fs::File::open(path).map_err(|error| CliLocalError::Io {
        path: Some(path.to_path_buf()),
        message: error.to_string(),
    })?;
    let mut bytes = Vec::new();
    file.take((MAX_RON_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| CliLocalError::Io {
            path: Some(path.to_path_buf()),
            message: error.to_string(),
        })?;
    if bytes.len() > MAX_RON_BYTES {
        return Err(CliLocalError::Io {
            path: Some(path.to_path_buf()),
            message: "RON input exceeds 32 MiB limit".into(),
        });
    }
    let source = String::from_utf8(bytes).map_err(|error| CliLocalError::Io {
        path: Some(path.to_path_buf()),
        message: error.to_string(),
    })?;
    ron::from_str(&source).map_err(|error| CliLocalError::Ron {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}
