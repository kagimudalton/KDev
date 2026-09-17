use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::paths::kdev_root;

#[derive(Serialize)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub source: String,
}

#[derive(Serialize, Deserialize, Default)]
struct ExtensionState {
    /// Extension ids the user has explicitly disabled. Anything not listed
    /// here is enabled by default, since KDev does not yet execute
    /// extension code -- this only tracks the user's stated intent so a
    /// future extension loader has something authoritative to read.
    #[serde(default)]
    disabled: HashSet<String>,
}

fn state_path() -> Result<PathBuf, String> {
    Ok(kdev_root()?.join("extensions").join(".kdev-extension-state.json"))
}

fn read_state() -> ExtensionState {
    let Ok(path) = state_path() else { return ExtensionState::default() };
    let Ok(text) = fs::read_to_string(path) else { return ExtensionState::default() };
    serde_json::from_str(&text).unwrap_or_default()
}

fn write_state(state: &ExtensionState) -> Result<(), String> {
    let path = state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create extension state directory: {e}"))?;
    }
    let bytes = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, bytes).map_err(|e| format!("cannot write extension state: {e}"))
}

/// Directories KDev will look in for extensions, in priority order: the
/// bundled resource directory and the folder beside the executable (both
/// cover a packaged/portable release that ships the repository's
/// `extensions/` folder), then the writable KDev data root, where a user
/// can drop in their own local extensions after install.
fn extension_dirs(app: &AppHandle) -> Vec<(PathBuf, &'static str)> {
    let mut dirs = Vec::new();
    if let Ok(resource_dir) = app.path().resource_dir() {
        dirs.push((resource_dir.join("extensions"), "bundled"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            dirs.push((exe_dir.join("extensions"), "bundled"));
        }
    }
    if let Ok(root) = kdev_root() {
        dirs.push((root.join("extensions"), "user-installed"));
    }
    dirs
}

#[tauri::command]
pub fn list_extensions(app: AppHandle) -> Result<Vec<ExtensionInfo>, String> {
    crate::security::require_unlocked()?;
    let state = read_state();
    let mut result: Vec<ExtensionInfo> = Vec::new();
    let mut seen: HashMap<String, ()> = HashMap::new();

    for (dir, source) in extension_dirs(&app) {
        if !dir.is_dir() {
            continue;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(id) = entry.file_name().to_str() {
                    if seen.insert(id.to_string(), ()).is_none() {
                        result.push(ExtensionInfo {
                            id: id.into(),
                            name: id.into(),
                            enabled: !state.disabled.contains(id),
                            source: source.into(),
                        });
                    }
                }
            }
        }
    }

    result.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(result)
}

/// Records whether an extension should be treated as enabled. This does
/// not load or execute extension code -- KDev has no extension runtime
/// yet -- it persists the user's choice so the UI is honest and so a
/// future loader has a real source of truth instead of always reporting
/// every discovered extension as enabled.
#[tauri::command]
pub fn set_extension_enabled(id: String, enabled: bool) -> Result<(), String> {
    crate::security::require_unlocked()?;
    if id.is_empty() || id.contains('/') || id.contains('\\') {
        return Err("invalid extension id".into());
    }
    let mut state = read_state();
    if enabled {
        state.disabled.remove(&id);
    } else {
        state.disabled.insert(id);
    }
    write_state(&state)
}
