# KDev Portable Runtime Layout

KDev can carry host-independent runtimes beside the application. The launcher selects a runtime by operating system and CPU architecture before falling back to the host when a bundled runtime is not present.

Expected layout:

- `runtime/windows-x64/`
- `runtime/windows-arm64/`
- `runtime/linux-x64/`
- `runtime/linux-arm64/`
- `runtime/macos-x64/`
- `runtime/macos-arm64/`

Runtime packages are intentionally not committed to the source repository. Release artifacts can package the appropriate licensed runtime components.
