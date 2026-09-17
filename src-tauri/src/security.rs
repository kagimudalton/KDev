use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;
use serde::Serialize;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::paths::kdev_root;

/// Whether this running process has been unlocked with the correct master
/// password. This is process-wide, not per-window/session-token, which is
/// appropriate for a single-user desktop app: KDev has exactly one window
/// and one workspace per process. Starting `false` means every launch
/// requires re-entering the master password when one is configured, even
/// though the frontend also shows a lock screen -- this flag is what
/// actually stops workspace commands from running, so the lock cannot be
/// bypassed by calling a Tauri command directly (e.g. from devtools)
/// without going through the frontend's unlock flow at all.
static UNLOCKED: AtomicBool = AtomicBool::new(false);

#[derive(Serialize)]
pub struct SecurityState {
    pub configured: bool,
    pub config_path: String,
}

/// The security configuration lives under KDev's own writable data root
/// (see `paths::kdev_root`) rather than a fixed per-machine location such
/// as `LOCALAPPDATA`. KDev is meant to travel on removable media, so a
/// master password created on one PC should still work after moving the
/// drive to another -- pinning it to one machine's local app-data folder
/// would silently break that.
fn config_path() -> Result<PathBuf, String> {
    Ok(kdev_root()?.join("security").join("security.json"))
}

/// Every workspace-affecting command should call this first. It is a no-op
/// (always `Ok`) when no master password has been configured, and only
/// blocks once a password exists and this process has not yet verified it.
pub fn require_unlocked() -> Result<(), String> {
    let path = config_path()?;
    if !path.is_file() {
        return Ok(());
    }
    if UNLOCKED.load(Ordering::SeqCst) {
        Ok(())
    } else {
        Err("KDev is locked. Unlock with your master password first.".into())
    }
}

#[tauri::command]
pub fn security_state() -> Result<SecurityState, String> {
    let path = config_path()?;
    Ok(SecurityState { configured: path.is_file(), config_path: path.to_string_lossy().into() })
}

#[tauri::command]
pub fn set_master_password(password: String) -> Result<(), String> {
    // If a password is already configured, changing it is itself a
    // workspace security operation and must not be possible while locked.
    require_unlocked()?;
    if password.chars().count() < 10 {
        return Err("Master password must contain at least 10 characters.".into());
    }
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::json!({"version": 1, "password_hash": hash});
    fs::write(&path, serde_json::to_vec_pretty(&data).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    // No prior password to satisfy (require_unlocked() only blocks once a
    // password file already exists), so this process is now the one that
    // just set it -- treat it as unlocked rather than immediately locking
    // the person out of the session they are already in.
    UNLOCKED.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn verify_master_password(password: String) -> Result<bool, String> {
    let text = fs::read_to_string(config_path()?).map_err(|e| format!("security configuration unavailable: {e}"))?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("invalid security configuration: {e}"))?;
    let hash = value
        .get("password_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "invalid security configuration".to_string())?;
    let parsed = PasswordHash::new(hash).map_err(|e| e.to_string())?;
    let ok = Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok();
    if ok {
        UNLOCKED.store(true, Ordering::SeqCst);
    }
    Ok(ok)
}

/// Re-locks the workspace for the rest of this process without restarting
/// KDev. Exposed so the UI can offer an explicit "Lock now" action.
#[tauri::command]
pub fn lock_workspace() -> Result<(), String> {
    UNLOCKED.store(false, Ordering::SeqCst);
    Ok(())
}
