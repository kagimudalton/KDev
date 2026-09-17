fn main() {
    // Embed a custom Windows application manifest so the built .exe
    // correctly declares Windows 10/11 compatibility (see
    // windows-app-manifest.xml) instead of falling back to Tauri's minimal
    // default manifest. This is Windows-only: non-Windows targets use
    // Tauri's normal build path unchanged and never reference the Windows
    // manifest file or WindowsAttributes API.
    #[cfg(windows)]
    {
        let windows_attributes = tauri_build::WindowsAttributes::new()
            .app_manifest(include_str!("windows-app-manifest.xml"));
        let attributes = tauri_build::Attributes::new().windows_attributes(windows_attributes);
        tauri_build::try_build(attributes).expect("failed to run the KDev Tauri build script (Windows)");
    }
    #[cfg(not(windows))]
    {
        tauri_build::build();
    }
}
