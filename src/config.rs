//! # Git Configuration Parsing
//!
//! This module handles the parsing of `.git/config` files to extract repository-local
//! settings, primarily focusing on user identity (name and email) for commits.

use std::path::Path;

/// Represents the top-level structure of a Git configuration file.
#[derive(serde::Deserialize)]
pub struct GitConfig {
    /// The `[user]` section of the config.
    pub user: UserConfig,
}

/// Represents the `[user]` section containing identity information.
#[derive(serde::Deserialize)]
pub struct UserConfig {
    /// The user's name (e.g., "John Doe").
    pub name: String,
    /// The user's email address (e.g., "john@example.com").
    pub email: String,
}

/// Reads the repository's `.git/config` file and extracts the author's name and email.
///
/// If the config file is missing or malformed, it falls back to a default
/// "unknown_user" and "unknown@localhost" to ensure commit creation does not fail.
pub fn get_author(dir: &Path) -> (String, String) {
    let unknown_user = UserConfig {
        name: "unknown_user".to_string(),
        email: "unknown@localhost".to_string(),
    };
    let config_dir = dir.join(".git").join("config");
    let contents = std::fs::read_to_string(config_dir).unwrap_or_default();
    let parsed = toml::from_str(&contents).unwrap_or(GitConfig { user: unknown_user });
    (parsed.user.name, parsed.user.email)
}
