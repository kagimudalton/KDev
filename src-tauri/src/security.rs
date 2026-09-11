use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Serialize)]
pub struct SecurityState { pub configured: bool, pub config_path: String }

fn config_path() -> Result<PathBuf, String> {
    let root = std::env::current_exe().map_err(|e| e.to_string())?.parent().map(|p| p.to_path_buf()).ok_or_else(|| "cannot locate KDev root".to_string())?;
    Ok(root.join("config").join("security.json"))
}

fn fnv64(bytes: &[u8], seed: u64) -> u64 {
    let mut h = 14695981039346656037u64 ^ seed;
    for b in bytes { h ^= *b as u64; h = h.wrapping_mul(1099511628211); }
    h
}

fn digest(password: &str, salt: &str) -> String {
    let data = format!("KDev|{salt}|{password}");
    (0..8).map(|i| format!("{:016x}", fnv64(data.as_bytes(), i as u64 * 0x9e3779b97f4a7c15))).collect()
}

#[tauri::command]
pub fn security_state() -> Result<SecurityState, String> {
    let path = config_path()?;
    Ok(SecurityState { configured: path.is_file(), config_path: path.to_string_lossy().into() })
}

#[tauri::command]
pub fn set_master_password(password: String) -> Result<(), String> {
    if password.chars().count() < 10 { return Err("Master password must contain at least 10 characters.".into()); }
    let path = config_path()?;
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let salt = format!("{:016x}{:016x}", fnv64(password.as_bytes(), 0x1234), fnv64(password.as_bytes(), 0x5678));
    let verifier = digest(&password, &salt);
    let data = format!("{{\"version\":1,\"salt\":\"{salt}\",\"verifier\":\"{verifier}\"}}\n");
    fs::write(path, data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn verify_master_password(password: String) -> Result<bool, String> {
    let path = config_path()?;
    let text = fs::read_to_string(path).map_err(|e| format!("security configuration unavailable: {e}"))?;
    let salt = text.split("\"salt\":\"").nth(1).and_then(|x| x.split('\"').next()).ok_or_else(|| "invalid security configuration".to_string())?;
    let verifier = text.split("\"verifier\":\"").nth(1).and_then(|x| x.split('\"').next()).ok_or_else(|| "invalid security configuration".to_string())?;
    Ok(digest(&password, salt) == verifier)
}
