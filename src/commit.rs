use crate::object::write_object;
use std::time::{SystemTime, UNIX_EPOCH};
pub struct Signature {
    name: String,
    email: String,
    timestamp: u64,
    timezone: String,
}

pub struct Commit {
    tree: String,
    author: Signature,
    committer: Signature,
    message: String,
    parent: Option<String>,
}

fn get_timestamp() -> Result<u64, Box<dyn std::error::Error>> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(timestamp)
}

fn create_commit(
    tree_hash: String,
    commit_message: String,
    parent_hash: Option<String>,
) -> Result<Commit, Box<dyn std::error::Error>> {
    let time_stamp = get_timestamp()?;
    let author = Signature {
        name: "Fady".to_string(),
        email: "fady@test.com".to_string(),
        timestamp: time_stamp,
        timezone: "+0000".to_string(),
    };
    let commiter = Signature {
        name: author.name.clone(),
        email: author.email.clone(),
        timestamp: author.timestamp,
        timezone: author.timezone.clone(),
    };
    let commit = Commit {
        tree: tree_hash,
        author: author,
        committer: commiter,
        message: commit_message,
        parent: parent_hash,
    };
    Ok(commit)
}

fn write_commit(commit: &Commit) -> Result<String, Box<dyn std::error::Error>> {
    let mut serialized = Vec::new();
    write!(&mut serialized, "tree {}\n", commit.tree)?;
    if let Some(parent_hash) = &commit.parent {
        write!(&mut serialized, "parent {}\n", parent_hash)?;
    }
    write!(
        &mut serialized,
            "author {} <{}> {} {}\n",
        commit.author.name, commit.author.email, commit.author.timestamp, commit.author.timezone
    )?;
    write!(
        &mut serialized,
            "committer {} <{}> {} {}\n",
            commit.committer.name,
            commit.committer.email,
            commit.committer.timestamp,
            commit.committer.timezone
    )?;
    write!(&mut serialized, "\n{}\n", commit.message)?;
    let oid = write_object("commit", &serialized)?;
    Ok(oid)
}

pub fn write_commit_object(
    tree_hash: String,
    commit_message: String,
    parent_hash: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let commit = create_commit(tree_hash, commit_message, parent_hash)?;
    let commit_hash = write_commit(commit)?;
    Ok(commit_hash)
}
