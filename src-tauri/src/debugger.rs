use serde::Serialize;
use std::process::Command;
use tauri::AppHandle;

use crate::paths::kdev_root;
use crate::runtime::{resolve_tool, RuntimeSource};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugCapability {
    pub python: bool,
    pub python_source: String,
    pub javascript: bool,
    pub javascript_source: String,
    pub typescript: bool,
    pub note: String,
}

fn source_label(source: RuntimeSource) -> &'static str {
    match source {
        RuntimeSource::Bundled => "KDev bundled runtime",
        RuntimeSource::Host => "Host system",
        RuntimeSource::Missing => "Not available",
    }
}

#[tauri::command]
pub fn debugger_capability(app: AppHandle) -> Result<DebugCapability, String> {
    crate::security::require_unlocked()?;
    let root = kdev_root()?;

    let python_tool = resolve_tool(&app, &root, "python");
    let python = python_tool.source != RuntimeSource::Missing
        && Command::new(&python_tool.executable)
            .args(["-c", "import debugpy"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    let node_tool = resolve_tool(&app, &root, "node");
    let javascript = node_tool.source != RuntimeSource::Missing
        && Command::new(&node_tool.executable)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    Ok(DebugCapability {
        python,
        python_source: source_label(python_tool.source).into(),
        javascript,
        javascript_source: source_label(node_tool.source).into(),
        typescript: javascript,
        note: "Debugger adapters are discovered from the KDev runtime first, then the host machine. KDev does not require internet access for capability detection.".into(),
    })
}

#[tauri::command]
pub fn debug_python_command(app: AppHandle, project: String, file: String) -> Result<String, String> {
    crate::security::require_unlocked()?;
    if project.is_empty() || project.contains('/') || project.contains('\\') || file.contains("..") {
        return Err("invalid project or file path".into());
    }
    let root = kdev_root()?;
    let path = root.join("workspace/projects").join(&project).join(&file);
    if !path.is_file() || path.extension().and_then(|x| x.to_str()) != Some("py") {
        return Err("Python source file not found".into());
    }
    let python = resolve_tool(&app, &root, "python");
    if python.source == RuntimeSource::Missing {
        return Err("Python was not found in the KDev runtime or on the host PATH.".into());
    }
    Ok(format!(
        "{} -m debugpy --listen 127.0.0.1:5678 --wait-for-client \"{}\"",
        python.executable.display(),
        path.display()
    ))
}
