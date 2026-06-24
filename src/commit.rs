use crate::{config::get_author, object::write_object};
use std::{
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};
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
    tree_hash: &str,
    commit_message: &str,
    parent_hash: Option<&str>,
) -> Result<Commit, Box<dyn std::error::Error>> {
    let time_stamp = get_timestamp()?;
    let (name, email) = get_author();

    let author = Signature {
        name: name.clone(),
        email: email.clone(),
        timestamp: time_stamp,
        timezone: "+0000".to_string(),
    };
    let committer = Signature {
        name: name,
        email: email,
        timestamp: author.timestamp,
        timezone: author.timezone.clone(),
    };
    let commit = Commit {
        tree: tree_hash.to_owned(),
        author: author,
        committer: committer,
        message: commit_message.to_owned(),
        parent: parent_hash.map(|s| s.to_owned()),
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
    tree_hash: &str,
    commit_message: &str,
    parent_hash: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let commit = create_commit(tree_hash, commit_message, parent_hash)?;
    let commit_hash = write_commit(&commit)?;
    Ok(commit_hash)
}
