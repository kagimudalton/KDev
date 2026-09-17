use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// KDev is designed to run without installation, straight off a USB drive,
/// SD card, or other removable media (see README "Portable design"). To
/// keep that promise while also surviving a normal Windows installer,
/// every backend module resolves its writable data directory through this
/// single function instead of assuming `current_exe().parent()` is
/// writable.
///
/// Resolution order:
///   1. `<folder next to the executable>/KDevData` -- used whenever that
///      folder is actually writable. This is the true "portable" case:
///      KDev is running from a flash drive or an unpacked folder, and all
///      of its data travels with the drive.
///   2. A per-user writable directory (`%USERPROFILE%\Documents\KDev` on
///      Windows, `~/.kdev` elsewhere) -- used when KDev has been installed
///      into a protected location such as "Program Files", where writing
///      next to the executable would fail with "Access is denied. (OS
///      error 5)".
///
/// The result is cached for the lifetime of the process: the writability
/// probe touches the filesystem, and every command that needs the root
/// would otherwise repeat that probe on every single call.
pub fn kdev_root() -> Result<PathBuf, String> {
    static ROOT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    ROOT.get_or_init(resolve_kdev_root).clone()
}

fn resolve_kdev_root() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate KDev executable: {e}"))?;
    let exe_dir = exe
        .parent()
        .ok_or_else(|| "cannot determine KDev executable directory".to_string())?;
    let portable_root = exe_dir.join("KDevData");
    if is_writable_dir(&portable_root) {
        return Ok(portable_root);
    }
    user_data_root()
}

/// Best-effort writability probe: try to create the directory and write a
/// small marker file inside it. Used instead of checking permission bits,
/// since Windows ACL semantics don't map cleanly onto Unix-style mode bits
/// and the only reliable test is to actually attempt the write.
fn is_writable_dir(dir: &Path) -> bool {
    if fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".kdev-write-test");
    match fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

fn user_data_root() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let base = std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .ok_or_else(|| "USERPROFILE is unavailable".to_string())?;
        let root = base.join("Documents").join("KDev");
        fs::create_dir_all(&root).map_err(|e| format!("cannot create KDev workspace: {e}"))?;
        Ok(root)
    }
    #[cfg(not(windows))]
    {
        let base = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| "HOME is unavailable".to_string())?;
        let root = base.join(".kdev");
        fs::create_dir_all(&root).map_err(|e| format!("cannot create KDev workspace: {e}"))?;
        Ok(root)
    }
}
