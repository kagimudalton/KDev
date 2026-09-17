use std::path::Path;

#[cfg(windows)]
use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};

pub fn workspace_usage(root: &Path) -> Result<(u64, u64), String> {
    let mut used = 0u64;
    if root.exists() {
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                used = used.saturating_add(entry.metadata().map_err(|e| e.to_string())?.len());
            }
        }
    }

    let (_, free, _) = filesystem_space(root)?;
    Ok((used, free))
}

fn filesystem_space(path: &Path) -> Result<(u64, u64, u64), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

        let wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(once(0))
            .collect();
        let mut free_for_caller = 0u64;
        let mut total = 0u64;
        let mut free_total = 0u64;

        let ok = unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_for_caller,
                &mut total,
                &mut free_total,
            )
        };

        if ok == 0 {
            return Err(format!(
                "Windows could not determine filesystem capacity for {}: {}",
                path.display(),
                std::io::Error::last_os_error()
            ));
        }

        return Ok((total, free_for_caller, free_total));
    }

    #[cfg(unix)]
    {
        let output = std::process::Command::new("df")
            .args(["-Pk", &path.to_string_lossy()])
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().last().unwrap_or("");
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            return Err("Unable to determine filesystem capacity".into());
        }
        let total_kb = cols[1]
            .parse::<u64>()
            .map_err(|e| format!("invalid filesystem total: {e}"))?;
        let free_kb = cols[3]
            .parse::<u64>()
            .map_err(|e| format!("invalid filesystem free space: {e}"))?;
        let total = total_kb.saturating_mul(1024);
        let free = free_kb.saturating_mul(1024);
        return Ok((total, free, free));
    }

    #[allow(unreachable_code)]
    Err("Filesystem capacity is unsupported on this platform".into())
}

pub fn storage_snapshot(root: &Path) -> Result<String, String> {
    let (workspace_used, _) = workspace_usage(root)?;
    let (total, free, _) = filesystem_space(root)?;
    let used_filesystem = total.saturating_sub(free);
    let usage_percent = if total == 0 {
        0.0
    } else {
        (used_filesystem as f64 / total as f64) * 100.0
    };

    Ok(format!(
        "workspace_used_bytes={workspace_used}\nfree_bytes={free}\nfilesystem_total_bytes={total}\nfilesystem_usage_percent={usage_percent:.1}"
    ))
}
