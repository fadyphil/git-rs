use std::path::PathBuf;
use std::{fs, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RefsError {
    #[error("I/O error {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "HEAD contains a raw hash instead of a ref path.\nWARNING Detached HEAD state is not supported."
    )]
    DetachedHead,
}

pub fn read_head() -> Result<String, RefsError> {
    let path = fs::read_to_string(".git/HEAD")?;
    let clean_path = path
        .trim()
        .strip_prefix("ref: ")
        .ok_or(RefsError::DetachedHead)?;
    Ok(clean_path.to_string())
}

pub fn read_ref(path: &str) -> Result<Option<String>, RefsError> {
    let path = PathBuf::from(".git/").join(path);
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path)?;
    let cleaned = contents.trim();
    Ok(Some(cleaned.to_string()))
}

pub fn update_current_ref(new_head_commit_hash: &str) -> Result<(), RefsError> {
    let head_file_path = Path::new(".git").join(read_head()?);
    let finalized_hash = format!("{}\n", new_head_commit_hash);
    fs::write(head_file_path, finalized_hash)?;
    Ok(())
}
