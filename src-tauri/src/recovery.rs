use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::paths::kdev_root;

#[derive(Serialize, Deserialize)]
struct SnapshotEnvelope {
    relative_path: String,
    content: String,
    saved_at: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverySnapshot {
    pub project: String,
    pub relative_path: String,
    pub snapshot_path: String,
    pub size: u64,
    pub saved_at: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryContent {
    pub project: String,
    pub relative_path: String,
    pub content: String,
    pub saved_at: u64,
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn safe_component(name: &str) -> Result<(), String> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err("invalid project name".into());
    }
    Ok(())
}

fn recovery_dir(project: &str) -> Result<PathBuf, String> {
    safe_component(project)?;
    Ok(kdev_root()?
        .join("workspace/projects")
        .join(project)
        .join(".kdev")
        .join("recovery"))
}

fn safe_suffix(relative_path: &str) -> String {
    relative_path
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
        .collect()
}

/// Saves an in-editor snapshot of an unsaved file so it can be recovered
/// after a crash or an interrupted session. The snapshot stores the exact
/// original relative path alongside the content (not just a sanitized
/// filename), so it can be restored to the right place later instead of
/// only being visible as metadata.
#[tauri::command]
pub fn save_recovery_snapshot(project: String, relative_path: String, content: String) -> Result<String, String> {
    crate::security::require_unlocked()?;
    let dir = recovery_dir(&project)?;
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create recovery directory: {e}"))?;
    let saved_at = now_secs();
    let file_name = format!("{saved_at}-{}.json", safe_suffix(&relative_path));
    let path = dir.join(&file_name);
    let envelope = SnapshotEnvelope { relative_path: relative_path.clone(), content, saved_at };
    let bytes = serde_json::to_vec(&envelope).map_err(|e| e.to_string())?;
    fs::write(&path, bytes).map_err(|e| format!("cannot write recovery snapshot: {e}"))?;
    prune_old_snapshots(&dir, &relative_path, 5)?;
    Ok(path.to_string_lossy().replace('\\', "/"))
}

/// Keeps only the most recent `keep` snapshots for a given source file so
/// the recovery directory does not grow without bound over a long session.
fn prune_old_snapshots(dir: &Path, relative_path: &str, keep: usize) -> Result<(), String> {
    let suffix = format!("-{}.json", safe_suffix(relative_path));
    let mut matches: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(&suffix))
                .unwrap_or(false)
        })
        .collect();
    matches.sort();
    while matches.len() > keep {
        let oldest = matches.remove(0);
        let _ = fs::remove_file(oldest);
    }
    Ok(())
}

/// Lists recovery snapshots for one project, or every project when `project`
/// is omitted, newest first. Corrupt or unreadable snapshot files are
/// skipped rather than failing the whole listing.
#[tauri::command]
pub fn list_recovery_snapshots(project: Option<String>) -> Result<Vec<RecoverySnapshot>, String> {
    crate::security::require_unlocked()?;
    let root = kdev_root()?;
    let projects_root = root.join("workspace/projects");
    let mut result = Vec::new();

    let project_dirs: Vec<PathBuf> = match project {
        Some(p) if !p.is_empty() => {
            safe_component(&p)?;
            vec![projects_root.join(p)]
        }
        _ => {
            if !projects_root.is_dir() {
                return Ok(result);
            }
            fs::read_dir(&projects_root)
                .map_err(|e| e.to_string())?
                .flatten()
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| e.path())
                .collect()
        }
    };

    for project_dir in project_dirs {
        let project_name = project_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let dir = project_dir.join(".kdev").join("recovery");
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
            let metadata = match entry.metadata() {
                Ok(m) if m.is_file() => m,
                _ => continue,
            };
            let Ok(text) = fs::read_to_string(entry.path()) else { continue };
            let Ok(envelope) = serde_json::from_str::<SnapshotEnvelope>(&text) else { continue };
            result.push(RecoverySnapshot {
                project: project_name.clone(),
                relative_path: envelope.relative_path,
                snapshot_path: entry.path().to_string_lossy().replace('\\', "/"),
                size: metadata.len(),
                saved_at: envelope.saved_at,
            });
        }
    }

    result.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
    Ok(result)
}

/// Reads back the content of a recovery snapshot so it can be restored.
/// `snapshot_path` must resolve inside KDev's own writable root -- this is
/// the only command in KDev that accepts a raw filesystem path from the
/// frontend, so it is validated strictly to prevent it being used to read
/// arbitrary files elsewhere on disk.
#[tauri::command]
pub fn read_recovery_snapshot(snapshot_path: String) -> Result<RecoveryContent, String> {
    crate::security::require_unlocked()?;
    let root = kdev_root()?;
    let requested = PathBuf::from(&snapshot_path);
    let canonical_root = fs::canonicalize(&root).map_err(|e| format!("cannot resolve KDev root: {e}"))?;
    let canonical_path = fs::canonicalize(&requested).map_err(|_| "recovery snapshot not found".to_string())?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err("invalid recovery snapshot path".into());
    }
    if canonical_path.extension().and_then(|e| e.to_str()) != Some("json") {
        return Err("invalid recovery snapshot path".into());
    }
    let text = fs::read_to_string(&canonical_path).map_err(|e| format!("cannot read recovery snapshot: {e}"))?;
    let envelope: SnapshotEnvelope = serde_json::from_str(&text).map_err(|e| format!("corrupt recovery snapshot: {e}"))?;
    let project = canonical_path
        .parent() // recovery
        .and_then(|p| p.parent()) // .kdev
        .and_then(|p| p.parent()) // <project>
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    Ok(RecoveryContent {
        project,
        relative_path: envelope.relative_path,
        content: envelope.content,
        saved_at: envelope.saved_at,
    })
}
