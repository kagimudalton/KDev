use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Manager};

/// Every place KDev is willing to look for its own bundled copy of a tool,
/// in priority order, before it will even consider the host machine:
///   1. The Tauri resource directory (tools shipped via `bundle.resources`
///      in a packaged release).
///   2. A `runtime/` folder next to the running executable (the layout
///      documented in `runtime/README.md`, used for a portable, unpacked
///      distribution carried on removable media).
///   3. `runtime/` inside KDev's own writable data root, so a user or an
///      administrator can drop in an external runtime package after
///      install without repackaging the whole application (see "Portable
///      tooling": runtimes may ship as optional external assets).
fn runtime_roots(app: &AppHandle, kdev_root: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(resource_dir) = app.path().resource_dir() {
        roots.push(resource_dir.join("runtime"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            roots.push(exe_dir.join("runtime"));
        }
    }
    roots.push(kdev_root.join("runtime"));
    roots
}

fn platform_subdirs() -> &'static [&'static str] {
    if cfg!(windows) {
        &["windows-x64", "windows-arm64"]
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            &["macos-arm64", "macos-x64"]
        } else {
            &["macos-x64"]
        }
    } else if cfg!(target_arch = "aarch64") {
        &["linux-arm64"]
    } else {
        &["linux-x64"]
    }
}

fn bundled_executable(roots: &[PathBuf], tool: &str) -> Option<PathBuf> {
    let exe_name = if cfg!(windows) { format!("{tool}.exe") } else { tool.to_string() };
    for root in roots {
        for platform in platform_subdirs() {
            let candidate = root.join(platform).join("bin").join(&exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        let candidate = root.join("bin").join(&exe_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn host_tool_available(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSource {
    Bundled,
    Host,
    Missing,
}

impl RuntimeSource {
    fn label(self) -> &'static str {
        match self {
            RuntimeSource::Bundled => "KDev bundled runtime",
            RuntimeSource::Host => "Host system",
            RuntimeSource::Missing => "KDev runtime not installed",
        }
    }
}

pub struct ResolvedTool {
    pub executable: PathBuf,
    pub source: RuntimeSource,
}

/// Resolves the executable KDev should actually invoke for `tool`: a
/// bundled copy first, then a host copy, tracking which one was used so
/// every caller -- and ultimately the UI -- can be honest about where a
/// tool came from instead of implying everything is portable when it is
/// really just whatever happens to be on the Windows PATH.
pub fn resolve_tool(app: &AppHandle, kdev_root: &Path, tool: &str) -> ResolvedTool {
    let roots = runtime_roots(app, kdev_root);
    if let Some(path) = bundled_executable(&roots, tool) {
        return ResolvedTool { executable: path, source: RuntimeSource::Bundled };
    }
    if host_tool_available(tool) {
        return ResolvedTool { executable: PathBuf::from(tool), source: RuntimeSource::Host };
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
            tool: tool.into(),
            label: label.into(),
            ready: false,
            source: RuntimeSource::Missing,
            source_label: RuntimeSource::Missing.label().into(),
            detail: format!("{label} was not found in the KDev runtime directory or on the host PATH."),
        };
    }
    match Command::new(&resolved.executable).args(version_args).output() {
        Ok(o) if o.status.success() => {
            let mut text = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if text.is_empty() {
                text = String::from_utf8_lossy(&o.stderr).trim().to_string();
            }
            ToolStatus {
                tool: tool.into(),
                label: label.into(),
                ready: true,
                source: resolved.source,
                source_label: resolved.source.label().into(),
                detail: if text.is_empty() { "Ready".into() } else { text },
            }
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            ToolStatus {
                tool: tool.into(),
                label: label.into(),
                ready: false,
                source: resolved.source,
                source_label: format!("{} (not responding)", resolved.source.label()),
                detail: if err.is_empty() { "The tool did not report a version.".into() } else { err },
            }
        }
        Err(e) => ToolStatus {
            tool: tool.into(),
            label: label.into(),
            ready: false,
            source: RuntimeSource::Missing,
            source_label: RuntimeSource::Missing.label().into(),
            detail: e.to_string(),
        },
    }
}

/// The full Doctor report for every tool KDev's UI understands. Kept in one
/// place so the main Doctor panel, the Power Panel, and the Finish Center
/// all report identical, honest results instead of three different ad hoc
/// shell probes.
pub fn doctor_report(app: &AppHandle, kdev_root: &Path) -> Vec<ToolStatus> {
    vec![
        tool_status(app, kdev_root, "python", "Python", &["--version"]),
        tool_status(app, kdev_root, "node", "Node.js", &["--version"]),
        tool_status(app, kdev_root, "npm", "npm", &["--version"]),
        tool_status(app, kdev_root, "git", "Git", &["--version"]),
        tool_status(app, kdev_root, "ruff", "Ruff", &["--version"]),
        tool_status(app, kdev_root, "prettier", "Prettier", &["--version"]),
    ]
}
