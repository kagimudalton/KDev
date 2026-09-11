use std::fs;
use std::path::{Path, PathBuf};

pub fn workspace_usage(root: &Path) -> Result<(u64, u64), String> {
    let mut used = 0u64;
    if root.exists() {
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                used = used.saturating_add(entry.metadata().map_err(|e| e.to_string())?.len());
            }
        }
    }
    Ok((used, fs2_free_space(root).unwrap_or(0)))
}

fn fs2_free_space(path: &Path) -> Result<u64, String> {
    #[cfg(unix)]
    {
        let output = std::process::Command::new("df").args(["-Pk", &path.to_string_lossy()]).output().map_err(|e| e.to_string())?;
        let line = String::from_utf8_lossy(&output.stdout).lines().last().unwrap_or("");
        let cols: Vec<&str> = line.split_whitespace().collect();
        cols.get(3).and_then(|v| v.parse::<u64>().ok()).map(|kb| kb * 1024).ok_or_else(|| "Unable to determine free space".to_string())
    }
    #[cfg(windows)]
    {
        let root = path.components().next().map(|c| c.as_os_str().to_string_lossy().to_string()).unwrap_or_else(|| "C:".into());
        let drive = root.trim_end_matches('\\').trim_end_matches(':');
        let script = format!("(Get-PSDrive -Name '{}').Free", drive);
        let output = std::process::Command::new("powershell.exe").args(["-NoProfile", "-Command", &script]).output().map_err(|e| e.to_string())?;
        String::from_utf8_lossy(&output.stdout).trim().parse::<u64>().map_err(|e| e.to_string())
    }
}

pub fn storage_snapshot(root: &Path) -> Result<String, String> {
    let (used, free) = workspace_usage(root)?;
    Ok(format!("used_bytes={used}\nfree_bytes={free}"))
}
