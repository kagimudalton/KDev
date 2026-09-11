# KDev Testing Guide

KDev is being built as a portable desktop development environment. This guide is for hands-on testing and feedback.

## 1. Browser smoke test

From the repository root:

```powershell
npm install
npm run build
npm run dev
```

Open the Vite address shown by the terminal. Browser mode can exercise the UI, editor, project modal, online/offline indicator, and Linux workspace presentation. Native filesystem, terminal, WSL and bundled-runtime behavior require the Tauri desktop build.

## 2. Desktop development test

Prerequisites on Windows:

- Node.js 22+
- Rust toolchain
- Microsoft C++ build tools / Windows SDK as required by Tauri

Then:

```powershell
npm install
npm run tauri:dev
```

Run KDev from the removable-drive copy when testing portability. Do not test only from a fixed `C:\` development folder.

## 3. Core test matrix

### Workspace
- Launch KDev from a USB/SSD/SD-card copy.
- Close it, move the drive to another drive letter, and launch again.
- Confirm projects and settings still resolve relative to KDev.
- Create a Python, Web, React, Next.js, TypeScript and Empty project.

### Editor
- Open every starter file.
- Type, wait for autosave, close/reopen KDev, and confirm the edit remains.
- Test tabs, syntax highlighting, minimap, brackets and the Run button.
- Create a deliberate syntax error and record what diagnostics/autocomplete do.

### Terminal
- Run `pwd` / `Get-Location` as appropriate to the host shell.
- Run `python --version`, `node --version`, `npm --version`, and `git --version`.
- Run the current Python or JavaScript file.
- Run a command from inside a project and confirm the working directory is that project.

### Linux workspace
- Open `LINUX`.
- If WSL/Kali is installed, test `whoami`, `pwd`, `uname -a`, and `git --version`.
- If Linux is unavailable, confirm KDev clearly stays in UI/LAB mode instead of pretending a Linux environment exists.
- Security-learning tools should only be exercised against systems/labs you are authorized to test.

### KDev Doctor
- Open Tools → KDev Doctor.
- Run it online and offline.
- Record which host tools are detected and whether missing tools are reported clearly.

### Offline behavior
- Disconnect the internet.
- Edit and save files.
- Create/open projects.
- Use local runtimes that are actually installed/bundled.
- Confirm KDev does not claim that an internet connection is required for local editing/saving.

### Portable-storage stress
- Work from a USB/portable SSD.
- Save a file repeatedly.
- Watch for errors when the drive is nearly full or temporarily unavailable.
- Never intentionally disconnect storage while KDev is actively writing important project data; this test is only to observe graceful error handling if an accidental disconnect occurs.

## 4. Feedback format

For every problem, send:

1. **Feature:** e.g. Editor / Terminal / Linux / Project creation
2. **Exact action:** what you clicked/typed
3. **Expected:** what should happen
4. **Actual:** what happened
5. **Error text:** copy it exactly
6. **Host:** Windows/Linux/macOS/Android + version
7. **Storage:** USB flash / SSD / SD / internal disk
8. **Online:** yes/no
9. **KDev commit:** the commit currently being tested
10. **Screenshot/video:** attach if the problem is visual

## 5. High-value tests

Prioritize these first:

- Create project → edit → autosave → restart → verify data.
- Move KDev to another drive letter → verify projects still work.
- Run Python and JavaScript projects.
- Run Doctor and report every incorrect Ready/Check result.
- Test Linux Workspace with and without WSL/Kali.
- Disconnect internet and verify local editing/saving.
- Try deliberately malformed project/file names and report whether KDev rejects them safely.

Do not include passwords, tokens, private keys, or other secrets in bug reports.
