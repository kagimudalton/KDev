# KDev Managed Runtime Layout

KDev is designed to execute projects through its own managed runtime bundle. It does **not** silently fall back to the Windows host PATH.

For Windows x64 releases, the build pipeline supplies:

- `windows-x64/bin/` — KDev-owned Python and Node.js executables
- `windows-x64/node-global/` — npm-managed `tsx` and `prettier` plus their `.bin` launchers
- `windows-x64/git/` — complete portable Git tree, including its supporting DLLs
- `windows-x64/ruff/` — KDev-owned Ruff installation

Expected platform layout:

- `runtime/windows-x64/`
- `runtime/windows-arm64/`
- `runtime/linux-x64/`
- `runtime/linux-arm64/`
- `runtime/macos-x64/`
- `runtime/macos-arm64/`

The source repository keeps runtime binaries out of Git. GitHub Actions assembles the platform runtime during the release build and Tauri packages it into the Windows installer.

Kali Linux is different from a normal executable runtime: on Windows it runs through WSL2. KDev detects and uses a Kali WSL distribution rather than pretending a normal Windows binary is a Kali environment.
