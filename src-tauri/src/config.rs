use std::path::PathBuf;

use anyhow::{Context, Result};

include!(concat!(env!("OUT_DIR"), "/embedded_config.rs"));

const ENV_KEYS: &[&str] = &[
    "GOOGLE_CLIENT_ID",
    "GOOGLE_CLIENT_SECRET",
    "MICROSOFT_CLIENT_ID",
    "MICROSOFT_CLIENT_SECRET",
    "PUSH_RELAY_URL",
];

/// Load `.env` files and apply compile-time embedded defaults (from `build.rs`).
pub fn init_config() {
    load_env_files();
    apply_embedded_defaults();
}

pub fn env_var(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}

pub fn require_env(key: &str) -> Result<String> {
    env_var(key).with_context(|| format!("{key} not set"))
}

fn load_env_files() {
    for path in env_file_paths() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                parse_env_content(&content);
            }
            return;
        }
    }
}

fn env_file_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join(".env"));
        }
    }
    paths.push(PathBuf::from(".env"));
    paths.push(PathBuf::from("../.env"));
    paths
}

fn parse_env_content(content: &str) {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            if env_var(key).is_none() {
                std::env::set_var(key, value.trim().trim_matches('"'));
            }
        }
    }
}

fn apply_embedded_defaults() {
    for key in ENV_KEYS {
        if env_var(key).is_none() {
            if let Some(value) = embedded_value(key) {
                std::env::set_var(key, value);
            }
        }
    }
}
