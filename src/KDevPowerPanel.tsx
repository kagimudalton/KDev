import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./power-panel.css";

type Props = { onClose?: () => void };

const isWindows = () => navigator.platform.toLowerCase().includes("win");

export default function KDevPowerPanel({ onClose }: Props) {
  const [tab, setTab] = useState("runtime");
  const [output, setOutput] = useState("");
  const [busy, setBusy] = useState(false);

  async function command(cmd: string) {
    setBusy(true);
    try {
      setOutput(await invoke<string>("run_dev_command", { command: cmd, workingDirectory: undefined }));
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  }

  const shell = isWindows() ? "powershell" : "bash";
  const runtimeCommand = isWindows()
    ? "$tools='python','node','npm','git'; foreach($t in $tools){ try { $v=& $t --version 2>&1 | Select-Object -First 1; \"$t`t$v\" } catch { \"$t`tNOT FOUND\" } }"
    : "for t in python3 node npm git; do if command -v $t >/dev/null 2>&1; then printf '%s\\t%s\\n' $t \"$($t --version 2>&1 | head -n1)\"; else printf '%s\\tNOT FOUND\\n' $t; fi; done";
  const envCommand = isWindows()
    ? "Write-Output ('KDev root: ' + (Get-Location).Path); Write-Output ('OS: Windows'); Write-Output ('Architecture: ' + $env:PROCESSOR_ARCHITECTURE)"
    : "printf 'KDev root: %s\\nOS: %s\\nArchitecture: %s\\n' \"$PWD\" \"$(uname -s)\" \"$(uname -m)\"";
  const recoveryCommand = isWindows()
    ? "if(Test-Path '.kdev/recovery'){ Get-ChildItem '.kdev/recovery' -Recurse -File | Select-Object FullName,Length,LastWriteTime | Format-Table -AutoSize | Out-String } else { 'No recovery snapshots found.' }"
    : "if [ -d .kdev/recovery ]; then find .kdev/recovery -type f -printf '%p\\t%s bytes\\n'; else echo 'No recovery snapshots found.'; fi";
  const historyCommand = isWindows()
    ? "if(Get-Command git -ErrorAction SilentlyContinue){ git log --oneline --decorate -12 } else { 'Git is not installed.' }"
    : "if command -v git >/dev/null 2>&1; then git log --oneline --decorate -12; else echo 'Git is not installed.'; fi";

  function runTab() {
    const map: Record<string, string> = { runtime: runtimeCommand, environment: envCommand, recovery: recoveryCommand, history: historyCommand };
    void command(map[tab]);
  }

  return <div className="power-overlay">
    <section className="power-panel">
      <header className="power-header">
        <div><span>KDEV POWER TOOLS</span><h2>Workspace Control</h2></div>
        {onClose && <button onClick={onClose}>×</button>}
      </header>
      <div className="power-tabs">
        {[["runtime","Runtime"],["environment","Environment"],["recovery","Recovery"],["history","History"]].map(([id,label]) => <button key={id} className={tab === id ? "active" : ""} onClick={() => setTab(id)}>{label}</button>)}
      </div>
      <main className="power-main">
        {tab === "runtime" && <><h3>Runtime Manager</h3><p>Detect the development tools available on this machine. Bundled KDev runtimes will take priority in the portable distribution.</p><div className="power-grid"><article><b>Python</b><span>Portable-ready</span></article><article><b>Node.js / npm</b><span>Portable-ready</span></article><article><b>Git</b><span>Portable-ready</span></article><article><b>Formatters</b><span>Prettier + Ruff</span></article></div></>}
        {tab === "environment" && <><h3>Environment Manager</h3><p>Keep project tooling isolated from the host where the portable runtime is available.</p><ul><li>Per-project runtime selection</li><li>Environment diagnostics</li><li>PATH isolation for KDev tools</li><li>Offline-capable configuration</li></ul></>}
        {tab === "recovery" && <><h3>Recovery Center</h3><p>Crash and autosave snapshots live outside the normal project files so recovery can happen after an interrupted session.</p><button className="power-primary" onClick={runTab} disabled={busy}>Scan Recovery</button></>}
        {tab === "history" && <><h3>Version History</h3><p>When a project is a Git repository, KDev can expose its local commit history without requiring an online connection.</p><button className="power-primary" onClick={runTab} disabled={busy}>Read Local History</button></>}
        <div className="power-output"><div><b>{busy ? "Working…" : shell}</b><button onClick={() => setOutput("")}>Clear</button></div><pre>{output || "Run a diagnostic to see results here."}</pre></div>
      </main>
    </section>
  </div>;
}
