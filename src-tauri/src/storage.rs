use std::path::Path;

pub fn workspace_usage(root: &Path) -> Result<(u64, u64), String> {
    let mut used = 0u64;
    if root.exists() {
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                used = used.saturating_add(entry.metadata().map_err(|e| e.to_string())?.len());
            }
        }
    }
    Ok((used, filesystem_free_space(root).unwrap_or(0)))
}

fn filesystem_free_space(path: &Path) -> Result<u64, String> {
    #[cfg(unix)]
    {
        let output = std::process::Command::new("df")
            .args(["-Pk", &path.to_string_lossy()])
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let line = stdout.lines().last().unwrap_or("");
        let cols: Vec<&str> = line.split_whitespace().collect();
        cols.get(3)
            .and_then(|v| v.parse::<u64>().ok())
            .map(|kb| kb.saturating_mul(1024))
            .ok_or_else(|| "Unable to determine free space".to_string())
    }

    #[cfg(windows)]
    {
        let drive = path
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().trim_end_matches(':').trim_end_matches('\\').to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "C".into());
        let script = format!("(Get-PSDrive -Name '{}').Free", drive);
        let output = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
            .map_err(|e| e.to_string())?;
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u64>()
            .map_err(|e| e.to_string())
    }
}

pub fn storage_snapshot(root: &Path) -> Result<String, String> {
    let (workspace_used, free) = workspace_usage(root)?;
    let total = workspace_used.saturating_add(free);
    let usage_percent = if total == 0 { 0.0 } else { (workspace_used as f64 / total as f64) * 100.0 };
    Ok(format!(
        "workspace_used_bytes={workspace_used}\nfree_bytes={free}\nfilesystem_total_bytes={total}\nfilesystem_usage_percent={usage_percent:.1}"
    ))
}
