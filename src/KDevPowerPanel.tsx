import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./power-panel.css";

type Props = { onClose?: () => void };
type ToolStatus = { tool: string; label: string; ready: boolean; source: string; sourceLabel: string; detail: string };
type Recovery = { project: string; relativePath: string; snapshotPath: string; size: number; savedAt: number };

const isWindows = () => navigator.platform.toLowerCase().includes("win");

export default function KDevPowerPanel({ onClose }: Props) {
  const [tab, setTab] = useState("runtime");
  const [output, setOutput] = useState("");
  const [busy, setBusy] = useState(false);
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [recovery, setRecovery] = useState<Recovery[]>([]);

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

  async function loadRuntime() {
    setBusy(true);
    try { setTools(await invoke<ToolStatus[]>("doctor_report")); }
    catch (e) { setOutput(String(e)); }
    finally { setBusy(false); }
  }

  async function loadRecovery() {
    setBusy(true);
    try {
      const rows = await invoke<Recovery[]>("list_recovery_snapshots", {});
      setRecovery(rows);
      setOutput(rows.length ? `Found ${rows.length} recovery snapshot(s).` : "No recovery snapshots found.");
    } catch (e) { setOutput(String(e)); }
    finally { setBusy(false); }
  }

  async function restoreSnapshot(r: Recovery) {
    setBusy(true);
    try {
      const content = await invoke<{ relativePath: string; content: string }>("read_recovery_snapshot", { snapshotPath: r.snapshotPath });
      await invoke("write_workspace_file", { relativePath: content.relativePath, content: content.content });
      setOutput(`Restored ${content.relativePath}. Reopen the file in the editor to see the recovered content.`);
    } catch (e) { setOutput(String(e)); }
    finally { setBusy(false); }
  }

  const shell = isWindows() ? "powershell" : "bash";
  const historyCommand = isWindows()
    ? "if(Get-Command git -ErrorAction SilentlyContinue){ git log --oneline --decorate -12 } else { 'Git is not installed.' }"
    : "if command -v git >/dev/null 2>&1; then git log --oneline --decorate -12; else echo 'Git is not installed.'; fi";

  useEffect(() => { if (tab === "runtime") void loadRuntime(); if (tab === "recovery") void loadRecovery(); }, [tab]);

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
        {tab === "runtime" && <><h3>Runtime Manager</h3><p>KDev checks its own bundled runtime first, then the host machine, and always shows which one answered.</p><div className="power-grid">{tools.map(t => <article key={t.tool}><b>{t.label}</b><span>{t.ready ? t.sourceLabel : "Not found"}</span></article>)}</div><button className="power-primary" onClick={loadRuntime} disabled={busy}>Refresh</button></>}
        {tab === "environment" && <><h3>Environment Manager</h3><p>Keep project tooling isolated from the host where the portable runtime is available.</p><ul><li>Per-project runtime selection</li><li>Environment diagnostics</li><li>PATH isolation for KDev tools</li><li>Offline-capable configuration</li></ul></>}
        {tab === "recovery" && <><h3>Recovery Center</h3><p>Crash and autosave snapshots live outside the normal project files so recovery can happen after an interrupted session.</p><button className="power-primary" onClick={loadRecovery} disabled={busy}>Scan Recovery</button><div className="power-grid">{recovery.map(r => <article key={r.snapshotPath}><b>{r.project} / {r.relativePath}</b><span>{new Date(r.savedAt * 1000).toLocaleString()}</span><button onClick={() => void restoreSnapshot(r)} disabled={busy}>Restore</button></article>)}</div></>}
        {tab === "history" && <><h3>Version History</h3><p>When a project is a Git repository, KDev can expose its local commit history without requiring an online connection.</p><button className="power-primary" onClick={() => void command(historyCommand)} disabled={busy}>Read Local History</button></>}
        <div className="power-output"><div><b>{busy ? "Working…" : shell}</b><button onClick={() => setOutput("")}>Clear</button></div><pre>{output || "Run a diagnostic to see results here."}</pre></div>
      </main>
    </section>
  </div>;
}
