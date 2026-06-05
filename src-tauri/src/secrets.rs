use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SERVICE: &str = "screen-reminder";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

fn secrets_dir() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .or_else(dirs::home_dir)
        .context("resolve data dir")?
        .join("screen-reminder")
        .join("secrets");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn secret_path(kind: &str, account_id: &str) -> Result<PathBuf> {
    Ok(secrets_dir()?.join(format!("{kind}-{account_id}.json")))
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn store_raw(kind: &str, account_id: &str, value: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, &format!("{kind}:{account_id}"))?;
    entry.set_password(value).context("store secret in keychain")
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn load_raw(kind: &str, account_id: &str) -> Result<Option<String>> {
    let entry = keyring::Entry::new(SERVICE, &format!("{kind}:{account_id}"))?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn delete_raw(kind: &str, account_id: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, &format!("{kind}:{account_id}"))?;
    match entry.delete_password() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn store_raw(kind: &str, account_id: &str, value: &str) -> Result<()> {
    let path = secret_path(kind, account_id)?;
    fs::write(&path, value)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn load_raw(kind: &str, account_id: &str) -> Result<Option<String>> {
    let path = secret_path(kind, account_id)?;
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(fs::read_to_string(path)?))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn delete_raw(kind: &str, account_id: &str) -> Result<()> {
    let path = secret_path(kind, account_id)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn store_tokens(account_id: &str, tokens: &OAuthTokens) -> Result<()> {
    store_raw("oauth", account_id, &serde_json::to_string(tokens)?)
}

pub fn load_tokens(account_id: &str) -> Result<Option<OAuthTokens>> {
    Ok(match load_raw("oauth", account_id)? {
        Some(json) => Some(serde_json::from_str(&json)?),
        None => None,
    })
}

pub fn delete_tokens(account_id: &str) -> Result<()> {
    delete_raw("oauth", account_id)
}

pub fn store_password(account_id: &str, password: &str) -> Result<()> {
    store_raw("caldav", account_id, password)
}

pub fn load_password(account_id: &str) -> Result<Option<String>> {
    load_raw("caldav", account_id)
}

pub fn delete_password(account_id: &str) -> Result<()> {
    delete_raw("caldav", account_id)
}
