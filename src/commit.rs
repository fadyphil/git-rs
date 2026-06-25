//! # Git Commit Creation
//!
//! This module handles the construction and serialization of Git commit objects.
//! A commit object links a tree (the snapshot) with metadata such as the author,
//! committer, timestamp, and an optional parent commit to form the commit history (DAG).

use crate::{config::get_author, object::write_object};
use std::{
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

/// Errors that can occur during commit creation and serialization.
#[derive(Debug, Error)]
pub enum CommitError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("Object storage error: {0}")]
    Object(#[from] crate::object::ObjectError),
}

/// Represents a Git author or committer signature.
pub struct Signature {
    name: String,
    email: String,
    timestamp: u64,
    timezone: String,
}

/// Represents a Git commit object and its metadata.
pub struct Commit {
    tree: String,
    author: Signature,
    committer: Signature,
    message: String,
    parent: Option<String>,
}

/// Retrieves the current system time as a UNIX timestamp (seconds since epoch).
fn get_timestamp() -> Result<u64, CommitError> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(timestamp)
}

/// Constructs a `Commit` struct with the provided tree, message, and parent.
///
/// Author and committer information are read from the repository's `.git/config`
/// file, and the current system time is used for the timestamps.
fn create_commit(
    tree_hash: &str,
    commit_message: &str,
    parent_hash: Option<&str>,
    dir: &Path,
) -> Result<Commit, CommitError> {
    let time_stamp = get_timestamp()?;
    let (name, email) = get_author(dir);

    let author = Signature {
        name: name.clone(),
        email: email.clone(),
        timestamp: time_stamp,
        timezone: "+0000".to_string(),
    };
    let committer = Signature {
        name,
        email,
        timestamp: author.timestamp,
        timezone: author.timezone.clone(),
    };
    let commit = Commit {
        tree: tree_hash.to_owned(),
        author,
        committer,
        message: commit_message.to_owned(),
        parent: parent_hash.map(|s| s.to_owned()),
    };
    Ok(commit)
}

/// Serializes a `Commit` struct into the official Git ASCII format and writes
/// it to the object database. Returns the SHA-1 hash of the commit object.
fn write_commit(commit: &Commit, dir: &Path) -> Result<String, CommitError> {
    let mut serialized = Vec::new();
    writeln!(&mut serialized, "tree {}", commit.tree)?;
    if let Some(parent_hash) = &commit.parent {
        writeln!(&mut serialized, "parent {}", parent_hash)?;
    }
    writeln!(
        &mut serialized,
        "author {} <{}> {} {}",
        commit.author.name, commit.author.email, commit.author.timestamp, commit.author.timezone
    )?;
    writeln!(
        &mut serialized,
        "committer {} <{}> {} {}",
        commit.committer.name,
        commit.committer.email,
        commit.committer.timestamp,
        commit.committer.timezone
    )?;
    write!(&mut serialized, "\n{}\n", commit.message)?;
    let oid = write_object("commit", &serialized, dir)?;
    Ok(oid)
}

/// High-level function to create and write a commit object in one step.
///
/// This serves as the primary entry point for commit creation from the CLI dispatcher.
/// Returns the 40-character hex SHA-1 hash of the new commit object.
pub fn write_commit_object(
    tree_hash: &str,
    commit_message: &str,
    parent_hash: Option<&str>,
    dir: &Path,
) -> Result<String, CommitError> {
    let commit = create_commit(tree_hash, commit_message, parent_hash, dir)?;
    let commit_hash = write_commit(&commit, dir)?;
    Ok(commit_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::read_object;
    use std::fs;
    use tempfile::tempdir;

    fn setup_git_dir(dir: &std::path::Path) {
        fs::create_dir_all(dir.join(".git/objects")).unwrap();
        fs::create_dir_all(dir.join(".git/info")).unwrap();
        fs::create_dir_all(dir.join(".git/pack")).unwrap();
    }

    #[test]
    fn test_write_commit_no_parent() {
        let dir = tempdir().unwrap();
        setup_git_dir(dir.path());

        // Mock config
        fs::write(
            dir.path().join(".git/config"),
            "[user]\nname = \"Alice\"\nemail = \"alice@test.com\"\n",
        )
        .unwrap();

        let hash = write_commit_object("tree_hash_123", "my message", None, dir.path()).unwrap();
        assert_eq!(hash.len(), 40);

        let (kind, content) = read_object(&hash, dir.path()).unwrap();
        assert_eq!(kind, "commit");
        let text = String::from_utf8(content).unwrap();

        assert!(text.contains("tree tree_hash_123"));
        assert!(text.contains("author Alice <alice@test.com>"));
        assert!(text.contains("\nmy message\n"));
        assert!(
            !text.contains("parent"),
            "Root commit should NOT have a parent header"
        );
    }

    #[test]
    fn test_write_commit_with_parent() {
        let dir = tempdir().unwrap();
        setup_git_dir(dir.path());
        fs::write(
            dir.path().join(".git/config"),
            "[user]\nname = \"Bob\"\nemail = \"bob@test.com\"\n",
        )
        .unwrap();

        let hash = write_commit_object("tree_hash_123", "msg", Some("parent_hash_456"), dir.path())
            .unwrap();
        let (_, content) = read_object(&hash, dir.path()).unwrap();
        let text = String::from_utf8(content).unwrap();

        assert!(
            text.contains("parent parent_hash_456"),
            "Child commit MUST have parent header"
        );
    }

    #[test]
    fn test_commit_fallback_unknown_user() {
        let dir = tempdir().unwrap();
        setup_git_dir(dir.path());
        // Intentionally DO NOT create .git/config

        let hash = write_commit_object("tree_hash", "msg", None, dir.path()).unwrap();
        let (_, content) = read_object(&hash, dir.path()).unwrap();
        let text = String::from_utf8(content).unwrap();

        assert!(
            text.contains("unknown_user"),
            "Should fallback to unknown_user when config is missing"
        );
        assert!(text.contains("unknown@localhost"));
    }

    #[test]
    fn test_get_timestamp_is_modern() {
        let ts = get_timestamp().unwrap();
        // Assert it's greater than Jan 1, 2024
        assert!(ts > 1704067200);
    }
}
