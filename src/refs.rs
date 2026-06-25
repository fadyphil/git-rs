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

pub fn read_head(dir: &Path) -> Result<String, RefsError> {
    let path = dir.join(".git").join("HEAD");
    let contents = fs::read_to_string(path)?;
    let clean_path = contents
        .trim()
        .strip_prefix("ref: ")
        .ok_or(RefsError::DetachedHead)?;
    Ok(clean_path.to_string())
}

pub fn read_ref(path: &str, dir: &Path) -> Result<Option<String>, RefsError> {
    let path = dir.join(".git").join(path);
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path)?;
    let cleaned = contents.trim();
    Ok(Some(cleaned.to_string()))
}

pub fn update_current_ref(new_head_commit_hash: &str, dir: &Path) -> Result<(), RefsError> {
    let ref_path = read_head(dir)?;
    let head_file_path = dir.join(".git").join(ref_path);
    let finalized_hash = format!("{}\n", new_head_commit_hash);
    fs::write(head_file_path, finalized_hash)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_read_head_valid() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();

        let head = read_head(dir.path()).unwrap();
        assert_eq!(head, "refs/heads/main");
    }

    #[test]
    fn test_read_head_detached() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        fs::write(
            dir.path().join(".git/HEAD"),
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2\n",
        )
        .unwrap();

        let result = read_head(dir.path());
        assert!(matches!(result.unwrap_err(), RefsError::DetachedHead));
    }

    #[test]
    fn test_read_ref_exists() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/refs/heads")).unwrap();
        fs::write(dir.path().join(".git/refs/heads/main"), "hash123\n").unwrap();

        let r = read_ref("refs/heads/main", dir.path()).unwrap();
        assert_eq!(r, Some("hash123".to_string()));
    }

    #[test]
    fn test_read_ref_missing() {
        let dir = tempdir().unwrap();
        let r = read_ref("refs/heads/missing", dir.path()).unwrap();
        assert_eq!(r, None);
    }

    #[test]
    fn test_update_current_ref() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/refs/heads")).unwrap();
        fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        fs::write(dir.path().join(".git/refs/heads/main"), "old_hash\n").unwrap();

        update_current_ref("new_hash_123", dir.path()).unwrap();

        let content = fs::read_to_string(dir.path().join(".git/refs/heads/main")).unwrap();
        assert_eq!(content.trim(), "new_hash_123");
    }
}
