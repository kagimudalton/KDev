use serde::Serialize;
use std::{path::PathBuf, process::Command};

#[derive(Serialize)]
pub struct DebugCapability { pub python: bool, pub javascript: bool, pub typescript: bool, pub note: String }

fn root() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| e.to_string())?.parent().map(|p| p.to_path_buf()).ok_or_else(|| "cannot locate KDev root".into())
}

#[tauri::command]
pub fn debugger_capability() -> Result<DebugCapability, String> {
    let python = Command::new("python").args(["-c", "import debugpy"]).output().map(|o| o.status.success()).unwrap_or(false);
    let javascript = Command::new("node").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    let typescript = javascript;
    Ok(DebugCapability { python, javascript, typescript, note: "Debugger adapters are discovered locally; KDev does not require internet access for capability detection.".into() })
}

#[tauri::command]
pub fn debug_python_command(project: String, file: String) -> Result<String, String> {
    if project.is_empty() || project.contains('/') || project.contains('\\') || file.contains("..") { return Err("invalid project or file path".into()); }
    let path = root()?.join("workspace/projects").join(project).join(file);
    if !path.is_file() || path.extension().and_then(|x| x.to_str()) != Some("py") { return Err("Python source file not found".into()); }
    Ok(format!("python -m debugpy --listen 127.0.0.1:5678 --wait-for-client \"{}\"", path.display()))
}
