use serde::Serialize;
use std::{fs, path::PathBuf, process::{Command, Stdio}};

#[derive(Serialize)]
pub struct ProjectInfo { pub has_package_json: bool, pub has_requirements: bool, pub has_pyproject: bool, pub has_git: bool, pub scripts: Vec<String> }

fn root() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| e.to_string())?.parent().map(|p| p.to_path_buf()).ok_or_else(|| "cannot locate KDev root".into())
}
fn project_dir(project: &str) -> Result<PathBuf, String> {
    if project.is_empty() || project.contains('/') || project.contains('\\') || project == "." || project == ".." { return Err("invalid project name".into()); }
    Ok(root()?.join("workspace/projects").join(project))
}

#[tauri::command]
pub fn project_info(project: String) -> Result<ProjectInfo, String> {
    let dir = project_dir(&project)?;
    let package = dir.join("package.json");
    let mut scripts = Vec::new();
    if let Ok(text) = fs::read_to_string(&package) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(obj) = json.get("scripts").and_then(|v| v.as_object()) { scripts.extend(obj.keys().cloned()); }
        }
    }
    Ok(ProjectInfo { has_package_json: package.is_file(), has_requirements: dir.join("requirements.txt").is_file(), has_pyproject: dir.join("pyproject.toml").is_file(), has_git: dir.join(".git").exists(), scripts })
}

#[tauri::command]
pub fn install_project_dependencies(project: String) -> Result<String, String> {
    let dir = project_dir(&project)?;
    let (program, args): (&str, Vec<&str>) = if dir.join("package.json").is_file() { ("npm", vec!["install"]) }
        else if dir.join("requirements.txt").is_file() { ("python", vec!["-m", "pip", "install", "-r", "requirements.txt"]) }
        else { return Err("No package.json or requirements.txt found in this project.".into()); };
    let output = Command::new(program).args(args).current_dir(dir).stdin(Stdio::null()).output().map_err(|e| format!("cannot start dependency manager: {e}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() { text.push_str(&stderr); }
    if !output.status.success() { return Err(text); }
    Ok(text)
}

#[tauri::command]
pub fn start_web_preview(project: String, port: u16) -> Result<String, String> {
    let dir = project_dir(&project)?;
    if !dir.join("index.html").is_file() { return Err("Static preview requires an index.html in the project root.".into()); }
    let port = if port == 0 { 4173 } else { port };
    Command::new("python").args(["-m", "http.server", &port.to_string(), "--bind", "127.0.0.1"]).current_dir(dir).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|e| format!("cannot start preview server: {e}"))?;
    Ok(format!("http://127.0.0.1:{port}"))
}
