//! # Git Tree Serialization
//!
//! This module handles the recursive traversal of directories and the creation
//! of Git tree objects. Tree objects represent the state of a directory at a
//! specific point in time, storing the names, modes, and SHA-1 hashes of its
//! contents (files and subdirectories).

use std::{fs, io::Write, path::Path};

use crate::object::write_object;
use thiserror::Error;

/// Errors that can occur during tree serialization.
#[derive(Debug, Error)]
pub enum TreeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Object storage error: {0}")]
    Object(#[from] crate::object::ObjectError), // Wraps your custom ObjectError!

    #[error("Hex decoding error: {0}")]
    Hex(#[from] hex::FromHexError), // Wraps the hex crate error
}

/// Represents a single entry (file or directory) within a Git tree.
pub struct TreeEntry {
    /// The file mode (e.g., "100644" for files, "040000" for directories).
    pub mode: String,
    /// The name of the file or directory.
    pub name: String,
    /// The 40-character hex SHA-1 hash of the object.
    pub hash: String,
}

/// Recursively traverses a directory, hashes its contents, and writes tree objects to the database.
///
/// This function uses a post-order depth-first search approach, ensuring that child
/// objects (blobs and sub-trees) are written and hashed before their parent tree.
/// It returns the 40-character hex SHA-1 hash of the root tree object.
pub fn write_tree(path: &Path, dir: &Path) -> Result<String, TreeError> {
    let entries = fs::read_dir(path)?;
    let mut array_of_entries: Vec<TreeEntry> = Vec::new();

    for entry in entries {
        let entry = entry?;

        if entry.file_name() == ".git" {
            continue;
        }

        if entry.path().is_file() {
            let content = fs::read(entry.path())?;
            let object = write_object("blob", &content, dir)?;
            array_of_entries.push(TreeEntry {
                mode: "100644".to_string(),
                name: entry.file_name().to_string_lossy().into_owned(),
                hash: object,
            });
        } else if entry.path().is_dir() {
            let hashed_object = write_tree(&entry.path(), dir)?;
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
    Ok(write_object("tree", &formatted_tree_entries, dir)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::read_object;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_write_tree_empty_dir() {
        let dir = tempdir().unwrap();
        let hash = write_tree(dir.path(), dir.path()).unwrap();
        assert_eq!(hash.len(), 40);

        let (kind, content) = read_object(&hash, dir.path()).unwrap();
        assert_eq!(kind, "tree");
        assert!(
            content.is_empty(),
            "Empty directory should produce empty tree content"
        );
    }

    #[test]
    fn test_write_tree_sorting_and_modes() {
        let dir = tempdir().unwrap();
        // Arrange: Create files in reverse alphabetical order
        fs::write(dir.path().join("zebra.txt"), "z").unwrap();
        fs::write(dir.path().join("apple.txt"), "a").unwrap();
        fs::create_dir(dir.path().join("banana_dir")).unwrap();

        // Act
        let hash = write_tree(dir.path(), dir.path()).unwrap();

        // Assert: Read the raw bytes of the tree object
        let (kind, content) = read_object(&hash, dir.path()).unwrap();
        assert_eq!(kind, "tree");

        let text = String::from_utf8_lossy(&content);

        // Verify alphabetical sorting: apple -> banana_dir -> zebra
        let apple_pos = text.find("apple.txt").unwrap();
        let banana_pos = text.find("banana_dir").unwrap();
        let zebra_pos = text.find("zebra.txt").unwrap();

        assert!(apple_pos < banana_pos, "apple should come before banana");
        assert!(banana_pos < zebra_pos, "banana should come before zebra");

        // Verify modes
        assert!(text.contains("100644"), "Files should have 100644 mode");
        assert!(
            text.contains("040000"),
            "Directories should have 040000 mode"
        );
    }

    #[test]
    fn test_write_tree_ignores_git_dir() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("file.txt"), "data").unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main").unwrap();

        let hash = write_tree(dir.path(), dir.path()).unwrap();
        let (_, content) = read_object(&hash, dir.path()).unwrap();
        let text = String::from_utf8_lossy(&content);

        assert!(text.contains("file.txt"));
        assert!(!text.contains(".git"), "Tree should ignore .git directory");
    }
}
