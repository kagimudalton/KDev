use argon2::{password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use rand_core::OsRng;
use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Serialize)]
pub struct SecurityState { pub configured: bool, pub config_path: String }

fn config_path() -> Result<PathBuf, String> {
    #[cfg(windows)]
    let root = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).ok_or_else(|| "LOCALAPPDATA is unavailable".to_string())?;
    #[cfg(not(windows))]
    let root = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| "HOME is unavailable".to_string())?.join(".local").join("share");
    Ok(root.join("KDev").join("config").join("security.json"))
}

#[tauri::command]
pub fn security_state() -> Result<SecurityState, String> {
    let path = config_path()?;
    Ok(SecurityState { configured: path.is_file(), config_path: path.to_string_lossy().into() })
}

#[tauri::command]
pub fn set_master_password(password: String) -> Result<(), String> {
    if password.chars().count() < 10 { return Err("Master password must contain at least 10 characters.".into()); }
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt).map_err(|e| e.to_string())?.to_string();
    let path = config_path()?;
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let data = serde_json::json!({"version":1,"password_hash":hash});
    fs::write(path, serde_json::to_vec_pretty(&data).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn verify_master_password(password: String) -> Result<bool, String> {
    let text = fs::read_to_string(config_path()?).map_err(|e| format!("security configuration unavailable: {e}"))?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("invalid security configuration: {e}"))?;
    let hash = value.get("password_hash").and_then(|v| v.as_str()).ok_or_else(|| "invalid security configuration".to_string())?;
    let parsed = PasswordHash::new(hash).map_err(|e| e.to_string())?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
}
