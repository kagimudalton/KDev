use serde::Serialize;
use std::{fs, io::Write, path::{Path, PathBuf}, process::Command};
use walkdir::WalkDir;

mod git;
mod security;
mod runtime;
mod storage;

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
fn safe_project_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.len() > 80 { return Err("project name must be 1-80 characters".into()); }
    if name == "." || name == ".." || name.chars().any(|c| c == '/' || c == '\\' || c.is_control()) { return Err("invalid project name".into()); }
    Ok(name.to_string())
}
fn safe_relative_target(relative_path: &str) -> Result<PathBuf, String> { safe_workspace_path(relative_path) }

#[cfg(windows)]
fn wsl_distros() -> Vec<String> {
    Command::new("wsl.exe").args(["-l", "-q"]).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).lines().map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect()).unwrap_or_default()
}
#[cfg(windows)]
fn preferred_wsl_distro() -> Option<String> {
    let distros = wsl_distros();
    distros.iter().find(|d| d.to_lowercase().contains("kali")).cloned().or_else(|| distros.first().cloned())
}

#[tauri::command]
fn platform_info() -> Result<PlatformInfo, String> { Ok(PlatformInfo { os: std::env::consts::OS.into(), arch: std::env::consts::ARCH.into(), kdev_root: kdev_root()?.display().to_string() }) }

#[tauri::command]
fn linux_environment() -> Result<LinuxEnvironment, String> {
    #[cfg(target_os = "windows")]
    { let distros = wsl_distros(); if let Some(kali) = distros.iter().find(|d| d.to_lowercase().contains("kali")) { return Ok(LinuxEnvironment { available: true, provider: format!("WSL2 / {kali}"), detail: format!("Kali Linux distribution detected: {kali}") }); } if !distros.is_empty() { return Ok(LinuxEnvironment { available: true, provider: "WSL2".into(), detail: format!("Linux distribution detected: {}. Install Kali separately if you want the Kali environment.", distros.join(", ")) }); } let status = Command::new("wsl.exe").args(["--status"]).output(); if status.map(|o| o.status.success()).unwrap_or(false) { return Ok(LinuxEnvironment { available: false, provider: "WSL2".into(), detail: "WSL is installed, but no Linux distribution is registered.".into() }); } return Ok(LinuxEnvironment { available: false, provider: "WSL2".into(), detail: "WSL2 is not available on this host. The Linux workspace UI remains available.".into() }); }
    #[cfg(not(target_os = "windows"))]
    { let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into()); let available = Path::new(&shell).exists() || Path::new("/bin/bash").exists(); Ok(LinuxEnvironment { available, provider: "native shell".into(), detail: if available { format!("Shell detected: {shell}") } else { "No compatible shell was detected.".into() } }) }
}

#[cfg(windows)]
fn windows_path_to_wsl(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    let bytes = raw.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' { format!("/mnt/{}/{}", (bytes[0] as char).to_ascii_lowercase(), &raw[2..].trim_start_matches('/')) } else { raw }
}

#[tauri::command]
fn run_linux_command(command: String, working_directory: Option<String>) -> Result<String, String> {
    let command = command.trim(); if command.is_empty() { return Ok(String::new()); } if command.len() > 4096 { return Err("command is too long".into()); }
    #[cfg(target_os = "windows")]
    let output = {
        let distro = preferred_wsl_distro().ok_or_else(|| "No WSL Linux distribution is installed.".to_string())?;
        let cwd = match working_directory { Some(p) => windows_path_to_wsl(&safe_relative_target(&p)?), None => windows_path_to_wsl(&kdev_root()?) };
        let shell_command = format!("cd '{}' && {}", cwd.replace('\'', "'\\''"), command);
        Command::new("wsl.exe").args(["-d", &distro, "--", "bash", "-lc", &shell_command]).output().map_err(|e| format!("failed to start Linux shell: {e}"))?
    };
    #[cfg(not(target_os = "windows"))]
    let output = {
        let cwd = match working_directory { Some(p) => safe_relative_target(&p)?, None => kdev_root()? };
        Command::new("bash").args(["-lc", command]).current_dir(cwd).output().map_err(|e| format!("failed to start bash: {e}"))?
    };
    let mut text = String::from_utf8_lossy(&output.stdout).to_string(); let stderr = String::from_utf8_lossy(&output.stderr); if !stderr.is_empty() { text.push_str(&stderr); } if !output.status.success() && text.trim().is_empty() { text = format!("command exited with status {}", output.status); } Ok(text)
}

#[tauri::command]
fn run_dev_command(command: String, working_directory: Option<String>) -> Result<String, String> {
    let command = command.trim(); if command.is_empty() { return Ok(String::new()); } if command.len() > 4096 { return Err("command is too long".into()); }
    let cwd = match working_directory { Some(p) => safe_relative_target(&p)?, None => kdev_root()? };
    #[cfg(windows)]
    let output = Command::new("powershell.exe").args(["-NoProfile", "-NonInteractive", "-Command", command]).current_dir(cwd).output().map_err(|e| format!("failed to start PowerShell: {e}"))?;
    #[cfg(not(windows))]
    let output = Command::new("bash").args(["-lc", command]).current_dir(cwd).output().map_err(|e| format!("failed to start shell: {e}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string(); let stderr = String::from_utf8_lossy(&output.stderr); if !stderr.is_empty() { text.push_str(&stderr); } if !output.status.success() && text.trim().is_empty() { text = format!("command exited with status {}", output.status); } Ok(text)
}

#[tauri::command]
fn create_file(relative_path: String, content: String) -> Result<(), String> {
    let path = safe_workspace_path(&relative_path)?; if path.exists() { return Err("file or folder already exists".into()); }
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| format!("cannot create parent directory: {e}"))?; }
    let mut file = fs::File::create(&path).map_err(|e| format!("cannot create file: {e}"))?; file.write_all(content.as_bytes()).map_err(|e| format!("cannot write file: {e}"))?; file.sync_all().map_err(|e| format!("cannot flush file: {e}"))?; Ok(())
}
#[tauri::command]
fn create_folder(relative_path: String) -> Result<(), String> { let path = safe_workspace_path(&relative_path)?; if path.exists() { return Err("file or folder already exists".into()); } fs::create_dir_all(path).map_err(|e| format!("cannot create folder: {e}")) }
#[tauri::command]
fn delete_workspace_entry(relative_path: String) -> Result<(), String> {
    let path = safe_workspace_path(&relative_path)?; let normalized = relative_path.replace('\\', "/");
    if normalized == "workspace" || normalized == "workspace/projects" || normalized.ends_with("/projects") { return Err("protected workspace directory".into()); }
    if !path.exists() { return Err("entry does not exist".into()); }
    if path.is_dir() { fs::remove_dir_all(path).map_err(|e| format!("cannot delete folder: {e}")) } else { fs::remove_file(path).map_err(|e| format!("cannot delete file: {e}")) }
}
#[tauri::command]
fn rename_workspace_entry(relative_path: String, new_name: String) -> Result<String, String> {
    let path = safe_workspace_path(&relative_path)?; if !path.exists() { return Err("entry does not exist".into()); }
    let name = new_name.trim(); if name.is_empty() || name == "." || name == ".." || name.chars().any(|c| c == '/' || c == '\\' || c.is_control()) { return Err("invalid new name".into()); }
    let target = path.parent().ok_or_else(|| "cannot determine parent directory".to_string())?.join(name); if target.exists() { return Err("target already exists".into()); }
    fs::rename(&path, &target).map_err(|e| format!("cannot rename entry: {e}"))?;
    let root = kdev_root()?; Ok(target.strip_prefix(root).unwrap_or(&target).to_string_lossy().replace('\\', "/"))
}
#[tauri::command]
fn create_project(name: String, template: String) -> Result<Vec<String>, String> {
    let name = safe_project_name(&name)?; let root = kdev_root()?.join("workspace/projects").join(&name); if root.exists() { return Err(format!("project '{name}' already exists")); } fs::create_dir_all(&root).map_err(|e| format!("cannot create project: {e}"))?;
    let files: Vec<(&str, &str)> = match template.as_str() {
        "python" => vec![("main.py", "def main():\n    print(\"Hello from KDev\")\n\nif __name__ == \"__main__\":\n    main()\n"), ("README.md", "# KDev Python Project\n")],
        "web" => vec![("index.html", "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>KDev</title><link rel=\"stylesheet\" href=\"style.css\"></head><body><main><h1>Hello from KDev</h1><p>Your portable web project is ready.</p></main><script src=\"app.js\"></script></body></html>\n"), ("style.css", "body { font-family: system-ui, sans-serif; margin: 3rem; }\n"), ("app.js", "console.log(\"KDev web project ready\");\n"), ("README.md", "# KDev Web Project\n")],
        "react" => vec![("package.json", "{\n  \"name\": \"kdev-react-app\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {\"dev\": \"vite\", \"build\": \"vite build\"},\n  \"dependencies\": {\"@vitejs/plugin-react\": \"latest\", \"vite\": \"latest\", \"react\": \"latest\", \"react-dom\": \"latest\"}\n}\n"), ("index.html", "<div id=\"root\"></div><script type=\"module\" src=\"/src/main.jsx\"></script>\n"), ("src/main.jsx", "import React from 'react';\nimport { createRoot } from 'react-dom/client';\nimport './style.css';\nfunction App(){ return <h1>KDev React Project</h1>; }\ncreateRoot(document.getElementById('root')).render(<App />);\n"), ("src/style.css", "body { font-family: system-ui, sans-serif; margin: 3rem; }\n"), ("README.md", "# KDev React Project\n")],
        "next" => vec![("package.json", "{\n  \"name\": \"kdev-next-app\",\n  \"private\": true,\n  \"scripts\": {\"dev\": \"next dev\", \"build\": \"next build\", \"start\": \"next start\"},\n  \"dependencies\": {\"next\": \"latest\", \"react\": \"latest\", \"react-dom\": \"latest\"}\n}\n"), ("app/page.jsx", "export default function Page(){ return <main><h1>KDev Next.js Project</h1></main>; }\n"), ("app/layout.jsx", "export default function Layout({children}){ return <html><body>{children}</body></html>; }\n"), ("README.md", "# KDev Next.js Project\n")],
        "typescript" => vec![("main.ts", "const message: string = 'Hello from KDev TypeScript';\nconsole.log(message);\n"), ("README.md", "# KDev TypeScript Project\n")],
        _ => vec![("README.md", "# KDev Project\n\nCreated with KDev.\n")],
    };
    let mut created = Vec::new(); for (relative, content) in files { let path = root.join(relative); if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| format!("cannot create project directory: {e}"))?; } let mut file = fs::File::create(&path).map_err(|e| format!("cannot create {relative}: {e}"))?; file.write_all(content.as_bytes()).map_err(|e| format!("cannot write {relative}: {e}"))?; file.sync_all().map_err(|e| format!("cannot flush {relative}: {e}"))?; created.push(format!("workspace/projects/{name}/{relative}")); } Ok(created)
}
#[tauri::command]
fn write_workspace_file(relative_path: String, content: String) -> Result<(), String> {
    let path = safe_workspace_path(&relative_path)?; if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| format!("cannot create workspace directory: {e}"))?; } let temp = path.with_extension(format!("kdev-tmp-{}", std::process::id())); { let mut file = fs::File::create(&temp).map_err(|e| format!("cannot create temporary file: {e}"))?; file.write_all(content.as_bytes()).map_err(|e| format!("cannot write file: {e}"))?; file.sync_all().map_err(|e| format!("cannot flush file: {e}"))?; } #[cfg(windows)] if path.exists() { fs::remove_file(&path).map_err(|e| format!("cannot replace existing file: {e}"))?; } fs::rename(&temp, &path).map_err(|e| format!("cannot commit file atomically: {e}"))?; Ok(())
}
#[tauri::command]
fn read_workspace_file(relative_path: String) -> Result<String, String> { fs::read_to_string(safe_workspace_path(&relative_path)?).map_err(|e| format!("cannot read workspace file: {e}")) }
#[tauri::command]
fn list_workspace_files() -> Result<Vec<String>, String> { let kdev = kdev_root()?; let root = kdev.join("workspace"); if !root.exists() { return Ok(Vec::new()); } let mut files = Vec::new(); for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) { if entry.file_type().is_file() { if let Ok(path) = entry.path().strip_prefix(&kdev) { files.push(path.to_string_lossy().replace('\\', "/")); } } } files.sort(); Ok(files) }
#[tauri::command]
fn list_projects() -> Result<Vec<String>, String> { let root = kdev_root()?.join("workspace/projects"); if !root.exists() { return Ok(Vec::new()); } let mut projects = Vec::new(); for entry in fs::read_dir(root).map_err(|e| format!("cannot read projects: {e}"))?.filter_map(Result::ok) { if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { if let Some(name) = entry.file_name().to_str() { projects.push(name.to_string()); } } } projects.sort(); Ok(projects) }

#[tauri::command]
fn storage_status() -> Result<String, String> { storage::storage_snapshot(&kdev_root()?) }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() { tauri::Builder::default().invoke_handler(tauri::generate_handler![platform_info, linux_environment, run_linux_command, run_dev_command, create_file, create_folder, delete_workspace_entry, rename_workspace_entry, create_project, write_workspace_file, read_workspace_file, list_workspace_files, list_projects, git::git_status, security::security_state, security::set_master_password, security::verify_master_password, storage_status]).run(tauri::generate_context!()).expect("error while running KDev"); }