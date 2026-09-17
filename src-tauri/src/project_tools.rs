use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tauri::AppHandle;

use crate::paths::kdev_root;
use crate::runtime::{resolve_tool, RuntimeSource};

#[derive(Serialize)]
pub struct ProjectInfo {
    pub has_package_json: bool,
    pub has_requirements: bool,
    pub has_pyproject: bool,
    pub has_git: bool,
    pub scripts: Vec<String>,
}

fn project_dir(project: &str) -> Result<PathBuf, String> {
    if project.is_empty() || project.contains('/') || project.contains('\\') || project == "." || project == ".." {
        return Err("invalid project name".into());
    }
    Ok(kdev_root()?.join("workspace/projects").join(project))
}

#[tauri::command]
pub fn project_info(project: String) -> Result<ProjectInfo, String> {
    crate::security::require_unlocked()?;
    let dir = project_dir(&project)?;
    let package = dir.join("package.json");
    let mut scripts = Vec::new();
    if let Ok(text) = fs::read_to_string(&package) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(obj) = json.get("scripts").and_then(|v| v.as_object()) {
                scripts.extend(obj.keys().cloned());
            }
        }
    }
    Ok(ProjectInfo {
        has_package_json: package.is_file(),
        has_requirements: dir.join("requirements.txt").is_file(),
        has_pyproject: dir.join("pyproject.toml").is_file(),
        has_git: dir.join(".git").exists(),
        scripts,
    })
}

#[tauri::command]
pub fn install_project_dependencies(app: AppHandle, project: String) -> Result<String, String> {
    crate::security::require_unlocked()?;
    let dir = project_dir(&project)?;
    let root = kdev_root()?;
    let (tool, args): (&str, Vec<&str>) = if dir.join("package.json").is_file() {
        ("npm", vec!["install"])
    } else if dir.join("requirements.txt").is_file() {
        ("python", vec!["-m", "pip", "install", "-r", "requirements.txt"])
    } else {
        return Err("No package.json or requirements.txt found in this project.".into());
    };

    let resolved = resolve_tool(&app, &root, tool);
    if resolved.source == RuntimeSource::Missing {
        return Err(format!("{tool} was not found in the KDev runtime or on the host PATH."));
    }

    let output = Command::new(&resolved.executable)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot start dependency manager: {e}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        text.push_str(&stderr);
    }
    if !output.status.success() {
        return Err(text);
    }
    Ok(text)
}

#[tauri::command]
pub fn start_web_preview(app: AppHandle, project: String, port: u16) -> Result<String, String> {
    crate::security::require_unlocked()?;
    let dir = project_dir(&project)?;
    if !dir.join("index.html").is_file() {
        return Err("Static preview requires an index.html in the project root.".into());
    }
    let root = kdev_root()?;
    let python = resolve_tool(&app, &root, "python");
    if python.source == RuntimeSource::Missing {
        return Err("Python was not found in the KDev runtime or on the host PATH; it is required for the built-in static preview server.".into());
    }
    let port = if port == 0 { 4173 } else { port };
    Command::new(&python.executable)
        .args(["-m", "http.server", &port.to_string(), "--bind", "127.0.0.1"])
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot start preview server: {e}"))?;
    Ok(format!("http://127.0.0.1:{port}"))
}

fn run_tool_in_project(app: &AppHandle, dir: &std::path::Path, tool: &str, args: &[String]) -> Result<String, String> {
    let root = kdev_root()?;
    let resolved = resolve_tool(app, &root, tool);
    if resolved.source == RuntimeSource::Missing {
        return Err(format!("{tool} was not found in the KDev runtime or on the host PATH."));
    }
    let output = Command::new(&resolved.executable)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("cannot run {tool}: {e}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        text.push_str(&stderr);
    }
    if !output.status.success() {
        return Err(if text.trim().is_empty() { format!("{tool} exited with an error") } else { text });
    }
    Ok(text)
}

fn safe_relative_file(relative_path: &str) -> Result<(), String> {
    if relative_path.is_empty() || relative_path.contains("..") || Path::new(relative_path).is_absolute() {
        return Err("invalid file path".into());
    }
    Ok(())
}

/// Runs a project file through KDev's runtime manager (a bundled
/// interpreter first, the host's copy only as an explicit fallback) rather
/// than the frontend building a raw "python"/"node"/"npx" command string
/// that always resolves against whatever happens to be on the Windows
/// PATH.
#[tauri::command]
pub fn run_project_file(app: AppHandle, project: String, relative_path: String) -> Result<String, String> {
    crate::security::require_unlocked()?;
    safe_relative_file(&relative_path)?;
    let dir = project_dir(&project)?;
    let full = dir.join(&relative_path);
    if !full.is_file() {
        return Err("file not found".into());
    }
    let ext = full.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let (tool, args): (&str, Vec<String>) = match ext.as_str() {
        "py" => ("python", vec![relative_path.clone()]),
        "js" | "jsx" | "mjs" | "cjs" => ("node", vec![relative_path.clone()]),
        "ts" | "tsx" => ("tsx", vec![relative_path.clone()]),
        other => return Err(format!("No direct runner is configured for .{other} files. Use the terminal or a project script.")),
    };
    run_tool_in_project(&app, &dir, tool, &args)
}

/// Formats a project file through KDev's runtime manager, same rationale
/// as `run_project_file`.
#[tauri::command]
pub fn format_project_file(app: AppHandle, project: String, relative_path: String) -> Result<String, String> {
    crate::security::require_unlocked()?;
    safe_relative_file(&relative_path)?;
    let dir = project_dir(&project)?;
    let full = dir.join(&relative_path);
    if !full.is_file() {
        return Err("file not found".into());
    }
    let ext = full.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let (tool, args): (&str, Vec<String>) = if ext == "py" {
        ("ruff", vec!["format".into(), relative_path.clone()])
    } else if ["js", "jsx", "ts", "tsx", "html", "css", "json", "md"].contains(&ext.as_str()) {
        ("prettier", vec!["--write".into(), relative_path.clone()])
    } else {
        return Err(format!("No formatter is configured for .{ext} files."));
    };
    let output = run_tool_in_project(&app, &dir, tool, &args)?;
    Ok(if output.trim().is_empty() { "Formatted.".into() } else { output })
}
