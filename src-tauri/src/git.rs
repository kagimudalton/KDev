use serde::Serialize;
use std::{path::PathBuf, process::Command};

#[derive(Serialize)]
pub struct GitStatus { pub available: bool, pub branch: String, pub changed: Vec<String>, pub message: String }

fn root() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| e.to_string())?.parent().map(|p| p.to_path_buf()).ok_or_else(|| "cannot locate KDev root".into())
}

#[tauri::command]
pub fn git_status(project: Option<String>) -> Result<GitStatus, String> {
    let base = root()?;
    let cwd = project.map(|p| base.join("workspace/projects").join(p)).unwrap_or(base);
    let branch = Command::new("git").args(["branch", "--show-current"]).current_dir(&cwd).output();
    let status = Command::new("git").args(["status", "--short"]).current_dir(&cwd).output();
    match (branch, status) {
        (Ok(b), Ok(s)) if b.status.success() && s.status.success() => {
            let branch = String::from_utf8_lossy(&b.stdout).trim().to_string();
            let changed = String::from_utf8_lossy(&s.stdout).lines().map(str::to_string).filter(|x| !x.is_empty()).collect();
            Ok(GitStatus { available: true, branch: if branch.is_empty() { "(detached/no branch)".into() } else { branch }, changed, message: "Git repository detected".into() })
        }
        _ => Ok(GitStatus { available: false, branch: String::new(), changed: Vec::new(), message: "Git is not available here or this project is not a Git repository.".into() })
    }
}
