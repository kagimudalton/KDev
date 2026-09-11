# KDev

KDev is a portable, offline-first development environment designed to carry the development workspace, tooling and configuration with you.

## Product direction

- Portable workspace stored on the KDev drive by default
- Windows, Linux and macOS desktop targets with architecture-aware runtimes
- Android/iOS companion targets where the host operating system permits app execution
- React + TypeScript workbench
- Monaco-based code editor
- Integrated terminal and runtime orchestration
- Python, JavaScript, TypeScript, JSX/TSX, HTML and CSS workflows
- Local-first autosave and crash recovery
- Git integration
- Prettier/Ruff formatting
- Color picker with visual color swatches
- Project templates and project intelligence
- Optional online enhancements without making normal coding dependent on a network connection
- KDev profile, workspace personalization, security and licensing architecture

## Architecture

```text
KDev UI (React + TypeScript)
        |
        v
KDev Core / Tauri
        |
  +-----+-------------------+
  |         |        |       |
Editor   Workspace Runtime Recovery
  |         |        |       |
Monaco   Portable  Python   Atomic saves
         projects  Node      snapshots
                   Git

        |
        v
Portable storage
```

The workspace uses relative paths so the KDev drive can move between drive letters and compatible machines without rewriting project locations.

## Development

```bash
npm install
npm run tauri:dev
```

For a browser-only UI preview:

```bash
npm run dev
```

For a desktop build:

```bash
npm run tauri:build
```

## Design principles

1. Offline means usable, not crippled.
2. The user's source code remains theirs and remains exportable.
3. Projects live in the KDev workspace by default.
4. Network failures must not destroy local work.
5. Portable storage is treated as a first-class environment, not just a folder containing an installer.
6. Mature open-source components are preferred over reimplementing editors and language tooling from scratch.
7. Security, recovery, portability and personalization are architectural concerns.

## Developer

**KAGIMU DALTON**

KDev is built and developed by KAGIMU DALTON.

## Status

The repository currently contains the initial runnable workbench foundation: Tauri desktop shell, React/TypeScript UI, Monaco editor, portable workspace commands, platform detection and atomic local saves. Runtime bundling, language servers, Git UI, package managers, recovery UI, licensing, security vault and additional platform targets are part of the complete product implementation.
