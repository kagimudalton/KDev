import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./finish-center.css";

type Props = { onClose: () => void };
type Runtime = { tool: string; label: string; ready: boolean; source: string; sourceLabel: string; detail: string };
type Storage = { used: number; free: number; total: number; percent: number };
type Recovery = { project: string; relativePath: string; snapshotPath: string; size: number; savedAt: number };

const fmt = (n: number) => {
  if (!Number.isFinite(n)) return "—";
  const u = ["B", "KB", "MB", "GB", "TB"];
  let i = 0, v = n;
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; }
  return `${v.toFixed(v >= 100 ? 0 : v >= 10 ? 1 : 2)} ${u[i]}`;
};

export default function FinishCenter({ onClose }: Props) {
  const [tab, setTab] = useState("intelligence");
  const [output, setOutput] = useState("");
  const [busy, setBusy] = useState(false);
  const [runtimes, setRuntimes] = useState<Runtime[]>([]);
  const [storage, setStorage] = useState<Storage | null>(null);
  const [recovery, setRecovery] = useState<Recovery[]>([]);

  async function run(command: string) {
    setBusy(true);
    try { setOutput(await invoke<string>("run_dev_command", { command, workingDirectory: undefined })); }
    catch (e) { setOutput(String(e)); }
    finally { setBusy(false); }
  }

  async function refresh() {
    try {
      const report = await invoke<Runtime[]>("doctor_report");
      setRuntimes(report);
    } catch (e) {
      setRuntimes([]);
      setOutput(String(e));
    }
    try {
      const raw = await invoke<string>("storage_status");
      const values: Record<string, number> = {};
      raw.split("\n").forEach(line => { const [k, v] = line.split("="); if (k && v) values[k] = Number(v); });
      const used = values.workspace_used_bytes ?? 0, free = values.free_bytes ?? 0, total = values.filesystem_total_bytes ?? used + free;
      setStorage({ used, free, total, percent: values.filesystem_usage_percent ?? (total ? (used / total) * 100 : 0) });
    } catch {}
  }

  async function scanRecovery() {
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

  useEffect(() => { void refresh(); }, []);

  const tabs: [string, string][] = [
    ["intelligence", "IntelliSense"], ["runtime", "Runtimes"], ["recovery", "Recovery"], ["git", "Source Control"],
    ["preview", "Web Preview"], ["security", "Security"], ["extensions", "Extensions"], ["portable", "Portability"]
  ];

  return <div className="finish-overlay"><section className="finish-window">
    <header className="finish-header"><div><span>KDEV FINAL SYSTEMS</span><h2>Development Control Center</h2></div><button onClick={onClose}>×</button></header>
    <div className="finish-layout"><aside>{tabs.map(([id, label]) => <button key={id} className={tab === id ? "active" : ""} onClick={() => setTab(id)}>{label}</button>)}</aside>
      <main>
        {tab === "intelligence" && <section><div className="finish-hero"><span>CODE INTELLIGENCE</span><h3>Write code. KDev understands the project.</h3><p>Monaco provides syntax-aware completion, symbols, bracket intelligence, color decorators, folding, minimap and navigation. Project language services can be supplied by the portable runtime.</p></div><div className="feature-grid"><article><b>Python</b><span>Syntax + completion foundation</span></article><article><b>TypeScript / TSX</b><span>Type-aware editor foundation</span></article><article><b>Web</b><span>HTML + CSS + JavaScript</span></article><article><b>Ghost text</b><span>Optional AI layer — never required</span></article></div><button className="primary" onClick={() => void run("echo KDev code intelligence ready")}>Self-check</button></section>}
        {tab === "runtime" && <section><h3>Portable Runtime Manager</h3><p className="muted">KDev prefers runtimes shipped beside the application. Host detection is used as a development fallback, and the source is always shown.</p><div className="runtime-list">{runtimes.map(x => <div key={x.tool}><span className={x.ready ? "ok-dot" : "bad-dot"}></span><b>{x.label}</b><code>{x.sourceLabel}</code></div>)}</div><button className="primary" disabled={busy} onClick={() => void refresh()}>Refresh runtimes</button></section>}
        {tab === "recovery" && <section><h3>Recovery Center</h3><p className="muted">Snapshots are stored locally. Network access is never required for recovery.</p><button className="primary" disabled={busy} onClick={() => void scanRecovery()}>Scan recovery</button><div className="snapshot-list">{recovery.map(x => <div key={x.snapshotPath}><b>{x.project} / {x.relativePath}</b><span>{fmt(x.size)} · {new Date(x.savedAt * 1000).toLocaleString()}</span><button onClick={() => void restoreSnapshot(x)} disabled={busy}>Restore</button></div>)}</div></section>}
        {tab === "git" && <section><h3>Source Control</h3><p className="muted">Git operations remain local-first. Use the integrated terminal for commits, branches and remotes.</p><div className="button-grid"><button onClick={() => void run("git status --short")}>Working tree</button><button onClick={() => void run("git branch --show-current")}>Current branch</button><button onClick={() => void run("git log --oneline --decorate -12")}>History</button><button onClick={() => void run("git diff --stat")}>Diff summary</button></div></section>}
        {tab === "preview" && <section><h3>Web Preview</h3><p className="muted">Static projects can run locally without an internet connection.</p><button className="primary" onClick={() => void run("python -m http.server 4173 --bind 127.0.0.1")}>Start local preview command</button><p className="notice">For project-aware preview, use System Center → Project → Web Preview.</p></section>}
        {tab === "security" && <section><h3>Security & Trust</h3><div className="security-box"><b>Master password</b><span>Managed by KDev Security.</span></div><div className="security-box"><b>Workspace boundaries</b><span>Path traversal is rejected by the native workspace layer.</span></div><div className="security-box"><b>Extensions</b><span>Extensions must not silently access projects or external commands.</span></div><p className="notice">Security tooling in the Linux workspace is intended for systems and labs you are authorized to test.</p></section>}
        {tab === "extensions" && <section><h3>Extension Platform</h3><p className="muted">Extensions are isolated packages that can add tooling, themes, commands and integrations.</p><div className="feature-grid"><article><b>Local extensions</b><span>Loaded from the portable extensions directory</span></article><article><b>Permissions</b><span>Explicit scope for sensitive integrations</span></article><article><b>Offline</b><span>Installed extensions keep working without internet</span></article><article><b>Safe core</b><span>Core workspace remains independent</span></article></div></section>}
        {tab === "portable" && <section><h3>Portable Workspace</h3><p className="muted">KDev keeps workspace data beside the application and avoids hard-coded drive letters.</p><div className="portability-card"><span>Storage</span><b>{storage ? `${fmt(storage.free)} free / ${fmt(storage.total)}` : "checking…"}</b><small>{storage ? `${storage.percent.toFixed(1)}% filesystem usage` : ""}</small></div><div className="feature-grid"><article><b>Windows</b><span>Portable desktop bundle</span></article><article><b>Linux</b><span>Native shell / WSL integration</span></article><article><b>macOS</b><span>Native desktop target</span></article><article><b>Removable media</b><span>USB, portable SSD and SD workspace model</span></article></div></section>}
        {output && <div className="finish-output"><div><b>{busy ? "Working…" : "Output"}</b><button onClick={() => setOutput("")}>Clear</button></div><pre>{output}</pre></div>}
      </main>
    </div>
  </section></div>;
}
