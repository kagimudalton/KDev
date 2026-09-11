use serde::Serialize;
use std::{fs, io::Write, path::{Path, PathBuf}};
use walkdir::WalkDir;

#[derive(Serialize)]
struct PlatformInfo {
    os: String,
    arch: String,
    kdev_root: String,
}

fn kdev_root() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate KDev executable: {e}"))?;
    exe.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "cannot determine KDev root".to_string())
}

fn safe_workspace_path(relative_path: &str) -> Result<PathBuf, String> {
    let root = kdev_root()?;
    let relative = Path::new(relative_path);
    if relative.is_absolute() {
        return Err("absolute workspace paths are not allowed".into());
    }
    if relative.components().any(|component| matches!(component, std::path::Component::ParentDir)) {
        return Err("parent-directory traversal is not allowed".into());
    }
    Ok(root.join(relative))
}

#[tauri::command]
fn platform_info() -> Result<PlatformInfo, String> {
    Ok(PlatformInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        kdev_root: kdev_root()?.display().to_string(),
    })
}

#[tauri::command]
fn write_workspace_file(relative_path: String, content: String) -> Result<(), String> {
    let path = safe_workspace_path(&relative_path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create workspace directory: {e}"))?;
    }

    let temp = path.with_extension(format!("kdev-tmp-{}", std::process::id()));
    {
        let mut file = fs::File::create(&temp).map_err(|e| format!("cannot create temporary file: {e}"))?;
        file.write_all(content.as_bytes()).map_err(|e| format!("cannot write file: {e}"))?;
        file.sync_all().map_err(|e| format!("cannot flush file: {e}"))?;
    }
    fs::rename(&temp, &path).map_err(|e| format!("cannot commit file atomically: {e}"))?;
    Ok(())
}

#[tauri::command]
fn read_workspace_file(relative_path: String) -> Result<String, String> {
    let path = safe_workspace_path(&relative_path)?;
    fs::read_to_string(path).map_err(|e| format!("cannot read workspace file: {e}"))
}

#[tauri::command]
fn list_workspace_files() -> Result<Vec<String>, String> {
    let root = kdev_root()?.join("workspace");
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            if let Ok(path) = entry.path().strip_prefix(kdev_root()?) {
                files.push(path.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    files.sort();
    Ok(files)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            platform_info,
            write_workspace_file,
            read_workspace_file,
            list_workspace_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running KDev");
}
