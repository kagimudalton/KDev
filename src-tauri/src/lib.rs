use serde::Serialize;
use std::{fs, io::Write, path::{Path, PathBuf}, process::Command};
use walkdir::WalkDir;

#[derive(Serialize)]
struct PlatformInfo { os: String, arch: String, kdev_root: String }

#[derive(Serialize)]
struct LinuxEnvironment { available: bool, provider: String, detail: String }

fn kdev_root() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate KDev executable: {e}"))?;
    exe.parent().map(Path::to_path_buf).ok_or_else(|| "cannot determine KDev root".into())
}

fn safe_workspace_path(relative_path: &str) -> Result<PathBuf, String> {
    let root = kdev_root()?;
    let relative = Path::new(relative_path);
    if relative.is_absolute() { return Err("absolute workspace paths are not allowed".into()); }
    if relative.components().any(|c| matches!(c, std::path::Component::ParentDir)) { return Err("parent-directory traversal is not allowed".into()); }
    Ok(root.join(relative))
}

#[cfg(windows)]
fn wsl_distros() -> Vec<String> {
    Command::new("wsl.exe").args(["-l", "-q"]).output().ok().map(|o| {
        String::from_utf8_lossy(&o.stdout).lines().map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect()
    }).unwrap_or_default()
}

#[cfg(windows)]
fn preferred_wsl_distro() -> Option<String> {
    let distros = wsl_distros();
    distros.iter().find(|d| d.to_lowercase().contains("kali")).cloned().or_else(|| distros.first().cloned())
}

#[tauri::command]
fn platform_info() -> Result<PlatformInfo, String> {
    Ok(PlatformInfo { os: std::env::consts::OS.into(), arch: std::env::consts::ARCH.into(), kdev_root: kdev_root()?.display().to_string() })
}

#[tauri::command]
fn linux_environment() -> Result<LinuxEnvironment, String> {
    #[cfg(target_os = "windows")]
    {
        let distros = wsl_distros();
        if let Some(kali) = distros.iter().find(|d| d.to_lowercase().contains("kali")) {
            return Ok(LinuxEnvironment { available: true, provider: format!("WSL2 / {kali}"), detail: format!("Kali Linux distribution detected: {kali}") });
        }
        if !distros.is_empty() {
            return Ok(LinuxEnvironment { available: true, provider: "WSL2".into(), detail: format!("Linux distribution detected: {}. Install Kali separately if you want the Kali environment.", distros.join(", ")) });
        }
        let status = Command::new("wsl.exe").args(["--status"]).output();
        if status.map(|o| o.status.success()).unwrap_or(false) {
            return Ok(LinuxEnvironment { available: false, provider: "WSL2".into(), detail: "WSL is installed, but no Linux distribution is registered.".into() });
        }
        return Ok(LinuxEnvironment { available: false, provider: "WSL2".into(), detail: "WSL2 is not available on this host. The Linux workspace UI remains available.".into() });
    }

    #[cfg(not(target_os = "windows"))]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        let available = Path::new(&shell).exists() || Path::new("/bin/bash").exists();
        Ok(LinuxEnvironment { available, provider: "native shell".into(), detail: if available { format!("Shell detected: {shell}") } else { "No compatible shell was detected.".into() } })
    }
}

#[tauri::command]
fn run_linux_command(command: String) -> Result<String, String> {
    let command = command.trim();
    if command.is_empty() { return Ok(String::new()); }
    if command.len() > 4096 { return Err("command is too long".into()); }

    #[cfg(target_os = "windows")]
    let output = {
        let distro = preferred_wsl_distro().ok_or_else(|| "No WSL Linux distribution is installed. Open KDev Linux after installing a distribution.".to_string())?;
        Command::new("wsl.exe").args(["-d", &distro, "--", "bash", "-lc", command]).output().map_err(|e| format!("failed to start Linux shell through WSL: {e}"))?
    };

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("bash").args(["-lc", command]).output().map_err(|e| format!("failed to start bash: {e}"))?;

    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() { text.push_str(&stderr); }
    if !output.status.success() && text.trim().is_empty() { text = format!("command exited with status {}", output.status); }
    Ok(text)
}

#[tauri::command]
fn write_workspace_file(relative_path: String, content: String) -> Result<(), String> {
    let path = safe_workspace_path(&relative_path)?;
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| format!("cannot create workspace directory: {e}"))?; }
    let temp = path.with_extension(format!("kdev-tmp-{}", std::process::id()));
    { let mut file = fs::File::create(&temp).map_err(|e| format!("cannot create temporary file: {e}"))?; file.write_all(content.as_bytes()).map_err(|e| format!("cannot write file: {e}"))?; file.sync_all().map_err(|e| format!("cannot flush file: {e}"))?; }
    #[cfg(windows)]
    if path.exists() { fs::remove_file(&path).map_err(|e| format!("cannot replace existing file: {e}"))?; }
    fs::rename(&temp, &path).map_err(|e| format!("cannot commit file atomically: {e}"))?;
    Ok(())
}

#[tauri::command]
fn read_workspace_file(relative_path: String) -> Result<String, String> {
    fs::read_to_string(safe_workspace_path(&relative_path)?).map_err(|e| format!("cannot read workspace file: {e}"))
}

#[tauri::command]
fn list_workspace_files() -> Result<Vec<String>, String> {
    let kdev = kdev_root()?;
    let root = kdev.join("workspace");
    if !root.exists() { return Ok(Vec::new()); }
    let mut files = Vec::new();
    for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            if let Ok(path) = entry.path().strip_prefix(&kdev) { files.push(path.to_string_lossy().replace('\\', "/")); }
        }
    }
    files.sort();
    Ok(files)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![platform_info, linux_environment, run_linux_command, write_workspace_file, read_workspace_file, list_workspace_files])
        .run(tauri::generate_context!())
        .expect("error while running KDev");
}
