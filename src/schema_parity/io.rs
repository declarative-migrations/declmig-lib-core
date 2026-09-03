use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::ParityError;

const MAX_JSON_BYTES: u64 = 64 * 1024 * 1024;

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, ParityError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ParityError::InvalidPath)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_JSON_BYTES {
        return Err(ParityError::InvalidPath);
    }
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), ParityError> {
    if path.file_name().is_none() {
        return Err(ParityError::InvalidPath);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, bytes)?;
    fs::rename(temporary, path)?;
    Ok(())
}
