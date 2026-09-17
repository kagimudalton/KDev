use serde::Serialize;
use std::process::Command;
use tauri::AppHandle;

use crate::paths::kdev_root;
use crate::runtime::{resolve_tool, RuntimeSource};

#[derive(Serialize)]
pub struct GitStatus {
    pub available: bool,
    pub branch: String,
    pub changed: Vec<String>,
    pub message: String,
    pub source: String,
}

#[tauri::command]
pub fn git_status(app: AppHandle, project: Option<String>) -> Result<GitStatus, String> {
    crate::security::require_unlocked()?;
    let base = kdev_root()?;
    let cwd = match project {
        Some(p) if !p.is_empty() => base.join("workspace/projects").join(p),
        _ => base.clone(),
    };

    let git = resolve_tool(&app, &base, "git");
    if git.source == RuntimeSource::Missing {
        return Ok(GitStatus {
            available: false,
            branch: String::new(),
            changed: Vec::new(),
            message: "Git was not found in the KDev runtime or on the host PATH.".into(),
            source: "KDev runtime not installed".into(),
        });
    }

    let branch = Command::new(&git.executable).args(["branch", "--show-current"]).current_dir(&cwd).output();
    let status = Command::new(&git.executable).args(["status", "--short"]).current_dir(&cwd).output();
    let source_label = match git.source {
        RuntimeSource::Bundled => "KDev bundled runtime",
        RuntimeSource::Host => "Host system",
        RuntimeSource::Missing => "KDev runtime not installed",
    };

    match (branch, status) {
        (Ok(b), Ok(s)) if b.status.success() && s.status.success() => {
            let branch = String::from_utf8_lossy(&b.stdout).trim().to_string();
            let changed = String::from_utf8_lossy(&s.stdout)
                .lines()
                .map(str::to_string)
                .filter(|x| !x.is_empty())
                .collect();
            Ok(GitStatus {
                available: true,
                branch: if branch.is_empty() { "(detached/no branch)".into() } else { branch },
                changed,
                message: "Git repository detected".into(),
                source: source_label.into(),
            })
        }
        _ => Ok(GitStatus {
            available: false,
            branch: String::new(),
            changed: Vec::new(),
            message: "Git is available, but this workspace is not a Git repository.".into(),
            source: source_label.into(),
        }),
    }
}

fn project_cwd(project: &Option<String>) -> Result<std::path::PathBuf, String> {
    let base = kdev_root()?;
    Ok(match project {
        Some(p) if !p.is_empty() => base.join("workspace/projects").join(p),
        _ => base,
    })
}

fn run_git(app: &AppHandle, cwd: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let base = kdev_root()?;
    let git = resolve_tool(app, &base, "git");
    if git.source == RuntimeSource::Missing {
        return Err("Git was not found in the KDev runtime or on the host PATH.".into());
    }
    let output = Command::new(&git.executable)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        text.push_str(&stderr);
    }
    if !output.status.success() {
        return Err(if text.trim().is_empty() { format!("git {} failed", args.join(" ")) } else { text });
    }
    Ok(text)
}

/// Stages every change in the workspace (or project) and commits it. KDev
/// never pushes or commits without an explicit user action -- this command
/// only runs when the person clicks Commit in the Git panel.
#[tauri::command]
pub fn git_commit(app: AppHandle, project: Option<String>, message: String) -> Result<String, String> {
    crate::security::require_unlocked()?;
    let message = message.trim();
    if message.is_empty() {
        return Err("A commit message is required.".into());
    }
    let cwd = project_cwd(&project)?;
    run_git(&app, &cwd, &["add", "-A"])?;
    run_git(&app, &cwd, &["commit", "-m", message])
}

/// Pushes the current branch to its configured remote, if any.
#[tauri::command]
pub fn git_push(app: AppHandle, project: Option<String>) -> Result<String, String> {
    crate::security::require_unlocked()?;
    let cwd = project_cwd(&project)?;
    run_git(&app, &cwd, &["push"])
}
