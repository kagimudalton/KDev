# KDev

KDev is a portable, offline-first development environment designed to travel with your workspace and runtimes.

## Workspaces

KDev has two first-class workspaces:

- **Developer Workspace** — Monaco editor, project files, autosave, recovery, and the web/Python development stack.
- **Linux Workspace** — a Linux-themed Debian/Kali-inspired workspace with a terminal, Linux learning toolbox, networking/security-learning area, documentation entry points, and portable project access.

The Linux Workspace is designed to use a real Linux environment when the host provides one. On Windows, KDev can use WSL and will prefer an installed Kali Linux distribution when detected. Without WSL/Kali, the Linux UI remains available as an offline learning/workspace shell but does not pretend to be a full Kali installation.

Security tools should only be used against systems, networks, and labs you are authorized to test.

## Portable design

The workspace is drive-letter independent and keeps projects under KDev's portable workspace by default. Runtime and platform adapters are designed around the host OS and CPU architecture rather than assuming a fixed Windows installation path.

Target platforms include Windows, Linux, and macOS desktop environments. Mobile targets require platform-specific application/runtime integration; a Windows `.exe` cannot simply execute on Android or iOS from removable storage.

## Offline-first principles

KDev remains useful without internet access:

- edit code
- autosave locally
- recover after interrupted sessions
- use local development runtimes when bundled/available
- work with HTML/CSS/JavaScript and local projects
- use local Git
- use the Linux workspace and local shell when the host environment supports it

Internet access is only required for operations that inherently need a network, such as downloading packages, cloning remote repositories, remote GitHub operations, online documentation, AI features, and updates.

## Architecture

```text
KDev UI
   │
   ├── Developer Workspace
   │      ├── Editor
   │      ├── Explorer
   │      ├── Terminal
   │      └── Project tools
   │
   ├── Linux Workspace
   │      ├── Linux desktop shell
   │      ├── Bash/WSL adapter
   │      ├── Learning toolbox
   │      └── Security/networking lab entry points
   │
   └── KDev Core
          ├── Workspace manager
          ├── Runtime manager
          ├── Recovery manager
          ├── Platform adapters
          └── Portable storage
```

## Development

```bash
npm install
npm run tauri:dev
```

Browser-only UI development:

```bash
npm run dev
```

Production desktop build:

```bash
npm run tauri:build
```

## Design goals

- portable
- offline-first
- low-resource desktop shell
- drive-letter independent
- recovery-focused
- developer-oriented
- cross-platform architecture
- Linux/security learning without requiring KDev to impersonate a complete Kali distribution

## Developer

**KAGIMU DALTON** — creator and developer of KDev.

KDev is intended to be a practical development environment that can travel with the developer rather than requiring every host computer to be configured from scratch.
