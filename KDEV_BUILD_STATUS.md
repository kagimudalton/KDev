# KDev Build Status

KDev is being developed as one integrated build, followed by polish and fixes.

## Continuous-build checklist

- [x] Tauri desktop shell
- [x] Monaco editor
- [x] Developer workspace
- [x] Linux workspace with real-environment detection
- [x] Portable workspace paths
- [x] Project templates
- [x] Integrated terminal
- [x] File create/delete/rename
- [x] Autosave foundation
- [x] Recovery foundation
- [x] Formatting hooks
- [x] KDev Doctor foundation
- [x] Visual CSS color decorators
- [x] Cross-platform packaging workflows
- [x] Robust recovery review/restore UI (real snapshots with original path, list, and restore)
- [x] Nested explorer tree
- [x] Git/source-control UI (status, commit, push; read-only history was already present)
- [x] Web preview
- [x] Dependency manager UI
- [x] Workspace security/password vault (real lock screen gating the whole app when a master password is configured, backed by a process-wide backend lock so workspace/git/project/recovery commands cannot be invoked directly to bypass it)
- [x] Storage health monitoring
- [x] Release artifact verification (NSIS/MSI existence, checksums, embedded Windows manifest via mt.exe extraction, non-Windows build path unaffected)
- [x] File/formatter execution resolved through the runtime manager (bundled-first, host-fallback) instead of raw host command strings
- [x] Reproducible project templates (React/Next pin exact dependency versions; offline-dependency-install limitation stated explicitly in generated READMEs)
- [~] Extension system -- enable/disable is now real and persisted, but KDev still has no extension *execution* runtime; enabling one only records intent for a future loader. Labeled as management-only in the UI.
- [~] Runtime bundling -- Doctor honestly reports bundled vs. host tools and never mislabels a host tool as bundled. CI has an opt-in hook (`KDEV_RUNTIME_ASSETS_URL` repo variable) to fetch and bundle a real runtime package as a Tauri resource, but no runtime binaries are bundled by default; someone still has to build and publish that asset.
- [ ] Debugger integration -- capability detection and a launch-command generator exist, but there is no real debugger (breakpoints, stepping, variable inspection, call stack). Labeled as launch-capability-only in the UI. Needs a Debug Adapter Protocol client and editor integration, which is a substantial standalone feature.
- [ ] npm lockfile -- package-lock.json has not been generated or committed yet; this requires running `npm install` once from a machine/environment with real npm registry access. CI uses `npm ci` automatically once that lockfile exists, and falls back to `npm install` with a warning until then.

This file is intentionally a living engineering checklist and should be updated as integrated features land.
