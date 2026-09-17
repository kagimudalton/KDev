use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Manager};

fn runtime_roots(app: &AppHandle, kdev_root: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(resource_dir) = app.path().resource_dir() { roots.push(resource_dir.join("runtime")); }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() { roots.push(exe_dir.join("runtime")); }
    }
    roots.push(kdev_root.join("runtime"));
    roots
}

fn platform_subdirs() -> &'static [&'static str] {
    if cfg!(windows) { &["windows-x64", "windows-arm64"] }
    else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") { &["macos-arm64", "macos-x64"] } else { &["macos-x64"] }
    } else if cfg!(target_arch = "aarch64") { &["linux-arm64"] } else { &["linux-x64"] }
}

fn executable_names(tool: &str) -> Vec<String> {
    if cfg!(windows) { vec![format!("{tool}.exe"), format!("{tool}.cmd"), format!("{tool}.bat")] }
    else { vec![tool.to_string()] }
}

fn bundled_executable(roots: &[PathBuf], tool: &str) -> Option<PathBuf> {
    let names = executable_names(tool);
    for root in roots {
        for platform in platform_subdirs() {
            let p = root.join(platform);
            let dirs = [
                p.join("bin"),
                p.join("node-global"),
                p.join("node-global").join("bin"),
                p.join("node-global").join("node_modules").join(".bin"),
                p.join("git").join("cmd"),
                p.join("git").join("bin"),
                p.join("ruff").join("Scripts"),
            ];
            for dir in dirs {
                for name in &names {
                    let candidate = dir.join(name);
                    if candidate.is_file() { return Some(candidate); }
                }
            }
        }
        let dirs = [
            root.join("bin"),
            root.join("node-global"),
            root.join("node-global").join("bin"),
            root.join("node-global").join("node_modules").join(".bin"),
            root.join("git").join("cmd"),
            root.join("ruff").join("Scripts"),
        ];
        for dir in dirs {
            for name in &names {
                let candidate = dir.join(name);
                if candidate.is_file() { return Some(candidate); }
            }
        }
    }
    None
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSource {
    Bundled,
    /// Kept for compatibility with older state; new resolution never returns Host.
    Host,
    Missing,
}

impl RuntimeSource {
    fn label(self) -> &'static str {
        match self {
            RuntimeSource::Bundled => "KDev bundled runtime",
            RuntimeSource::Host => "Host system (disabled)",
            RuntimeSource::Missing => "KDev runtime not installed",
        }
    }
}

pub struct ResolvedTool { pub executable: PathBuf, pub source: RuntimeSource }

/// Resolve only KDev-owned binaries. There is intentionally no Windows PATH fallback.
pub fn resolve_tool(app: &AppHandle, kdev_root: &Path, tool: &str) -> ResolvedTool {
    let roots = runtime_roots(app, kdev_root);
    if let Some(path) = bundled_executable(&roots, tool) {
        return ResolvedTool { executable: path, source: RuntimeSource::Bundled };
    }
    ResolvedTool { executable: PathBuf::from(tool), source: RuntimeSource::Missing }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub tool: String,
    pub label: String,
    pub ready: bool,
    pub source: RuntimeSource,
    pub source_label: String,
    pub detail: String,
}

fn tool_status(app: &AppHandle, kdev_root: &Path, tool: &str, label: &str, version_args: &[&str]) -> ToolStatus {
    let resolved = resolve_tool(app, kdev_root, tool);
    if resolved.source == RuntimeSource::Missing {
        return ToolStatus {
            tool: tool.into(), label: label.into(), ready: false,
            source: RuntimeSource::Missing,
            source_label: RuntimeSource::Missing.label().into(),
            detail: format!("{label} is not installed in KDev's bundled runtime."),
        };
    }
    match Command::new(&resolved.executable).args(version_args).output() {
        Ok(o) if o.status.success() => {
            let mut text = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if text.is_empty() { text = String::from_utf8_lossy(&o.stderr).trim().to_string(); }
            ToolStatus {
                tool: tool.into(), label: label.into(), ready: true,
                source: resolved.source, source_label: resolved.source.label().into(),
                detail: if text.is_empty() { "Ready".into() } else { text },
            }
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            ToolStatus {
                tool: tool.into(), label: label.into(), ready: false,
                source: resolved.source, source_label: resolved.source.label().into(),
                detail: if err.is_empty() { "The bundled tool did not report a version.".into() } else { err },
            }
        }
        Err(e) => ToolStatus {
            tool: tool.into(), label: label.into(), ready: false,
            source: RuntimeSource::Missing, source_label: RuntimeSource::Missing.label().into(), detail: e.to_string(),
        },
    }
}

pub fn doctor_report(app: &AppHandle, kdev_root: &Path) -> Vec<ToolStatus> {
    vec![
        tool_status(app, kdev_root, "python", "Python", &["--version"]),
        tool_status(app, kdev_root, "node", "Node.js", &["--version"]),
        tool_status(app, kdev_root, "npm", "npm", &["--version"]),
        tool_status(app, kdev_root, "git", "Git", &["--version"]),
        tool_status(app, kdev_root, "ruff", "Ruff", &["--version"]),
        tool_status(app, kdev_root, "prettier", "Prettier", &["--version"]),
        tool_status(app, kdev_root, "tsx", "tsx", &["--version"]),
    ]
}
