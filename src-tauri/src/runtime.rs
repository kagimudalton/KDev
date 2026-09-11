use std::path::{Path, PathBuf};
use std::process::Command;

/// Portable runtime discovery. KDev prefers runtimes shipped beside the app,
/// then falls back to the host runtime so development remains usable before
/// the bundled-runtime distribution is installed.
pub fn runtime_root(kdev_root: &Path) -> PathBuf {
    kdev_root.join("runtime")
}

pub fn bundled_executable(kdev_root: &Path, tool: &str) -> Option<PathBuf> {
    let root = runtime_root(kdev_root);
    let candidates = if cfg!(windows) {
        vec![root.join("windows-x64").join("bin").join(format!("{tool}.exe")), root.join("bin").join(format!("{tool}.exe"))]
    } else {
        vec![root.join("bin").join(tool), root.join("linux-x64").join("bin").join(tool)]
    };
    candidates.into_iter().find(|p| p.is_file())
}

pub fn tool_version(kdev_root: &Path, tool: &str, args: &[&str]) -> Result<String, String> {
    let executable = bundled_executable(kdev_root, tool).unwrap_or_else(|| PathBuf::from(tool));
    let output = Command::new(&executable).args(args).output().map_err(|e| format!("{tool}: {e}"))?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() { Ok(if text.is_empty() { err } else { text }) }
    else { Err(if err.is_empty() { text } else { err }) }
}
