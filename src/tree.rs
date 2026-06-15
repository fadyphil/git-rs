use std::{fs, path::Path};

use crate::object::write_object;

pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: String,
}

pub fn hex_to_bytes(hex_content: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if hex_content.len() != 40 {
        return Err("Error hex must be exactly 40 chars long".into());
    }
    (0..hex_content.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_content[i..i + 2], 16).map_err(|e| e.into()))
        .collect()
}

pub fn write_tree(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let entries = fs::read_dir(path)?;
    let mut array_of_entries: Vec<TreeEntry> = Vec::new();

    for entry in entries {
        let entry = entry?;

        if entry.file_name() == ".git" {
            continue;
        }

        if entry.path().is_file() {
            let content = fs::read(entry.path())?;
            let object = write_object("blob", &content);
            array_of_entries.push(TreeEntry {
                mode: "100644".to_string(),
                name: entry.file_name().to_string_lossy().into_owned(),
                hash: object?,
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
    array_of_entries.sort_by_key(|k| k.name.clone());
    let mut formatted_tree_entries: Vec<u8> = Vec::new();
    for entry in array_of_entries {
        formatted_tree_entries.extend_from_slice(entry.mode.as_bytes());
        formatted_tree_entries.extend_from_slice(" ".as_bytes());
        formatted_tree_entries.extend_from_slice(entry.name.as_bytes());
        formatted_tree_entries.push(0x00); // Null terminator
        formatted_tree_entries.extend_from_slice(&hex_to_bytes(&entry.hash)?);
    }
    Ok(write_object("tree", &formatted_tree_entries)?)
}
