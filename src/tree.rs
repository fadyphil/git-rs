use std::{fs, io::Write, path::Path};

use crate::object::write_object;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TreeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Object storage error: {0}")]
    Object(#[from] crate::object::ObjectError), // Wraps your custom ObjectError!

    #[error("Hex decoding error: {0}")]
    Hex(#[from] hex::FromHexError), // Wraps the hex crate error
}
pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: String,
}

pub fn write_tree(path: &Path) -> Result<String, TreeError> {
    let entries = fs::read_dir(path)?;
    let mut array_of_entries: Vec<TreeEntry> = Vec::new();

    for entry in entries {
        let entry = entry?;

        if entry.file_name() == ".git" {
            continue;
        }

        if entry.path().is_file() {
            let content = fs::read(entry.path())?;
            let object = write_object("blob", &content)?;
            array_of_entries.push(TreeEntry {
                mode: "100644".to_string(),
                name: entry.file_name().to_string_lossy().into_owned(),
                hash: object,
            });
        } else if entry.path().is_dir() {
            let hashed_object = write_tree(&entry.path())?;
            array_of_entries.push(TreeEntry {
                mode: "040000".to_string(),
                name: entry.file_name().to_string_lossy().into_owned(),
                hash: hashed_object,
            });
        }
    }
    array_of_entries.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    let mut formatted_tree_entries: Vec<u8> = Vec::new();
    for entry in array_of_entries {
        write!(
            &mut formatted_tree_entries,
            "{} {}\0",
            entry.mode, entry.name
        )?;
        formatted_tree_entries.extend_from_slice(&hex::decode(&entry.hash)?);
    }
    Ok(write_object("tree", &formatted_tree_entries)?)
}
