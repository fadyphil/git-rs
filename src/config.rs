use std::path::Path;

#[derive(serde::Deserialize)]
pub struct GitConfig {
    pub user: UserConfig,
}
#[derive(serde::Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub email: String,
}

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
