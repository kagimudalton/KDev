use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Serialize)]
pub struct ExtensionInfo { pub id: String, pub name: String, pub enabled: bool }

fn root() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| e.to_string())?.parent().map(|p| p.to_path_buf()).ok_or_else(|| "cannot locate KDev root".into())
}

#[tauri::command]
pub fn list_extensions() -> Result<Vec<ExtensionInfo>, String> {
    let dir = root()?.join("extensions");
    if !dir.exists() { return Ok(Vec::new()); }
    let mut result = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())?.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            if let Some(id) = entry.file_name().to_str() { result.push(ExtensionInfo { id: id.into(), name: id.into(), enabled: true }); }
        }
    }
    result.sort_by(|a,b| a.id.cmp(&b.id));
    Ok(result)
}
