use std::path::PathBuf;
use std::{fs, path::Path};

pub fn read_head() -> Result<String, Box<dyn std::error::Error>> {
    let path = fs::read_to_string(".git/HEAD")?;
    if !path.starts_with("ref: ") {
        return Err("HEAD has hash instead of path,\nWARNING this is a DETACHED HEAD".into());
    }
    let clean_path = path.trim().strip_prefix("ref: ").unwrap();
    Ok(clean_path.to_string())
}

pub fn read_ref(path: String) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let path = PathBuf::from(".git/").join(path);
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path)?;
    let cleaned = contents.trim();
    Ok(Some(cleaned.to_string()))
}

pub fn update_current_ref(new_head_commit_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let head_file_path = Path::new(".git").join(read_head()?);
    let finalized_hash = format!("{}\n", new_head_commit_hash);
    fs::write(head_file_path, finalized_hash)?;
    Ok(())
}
