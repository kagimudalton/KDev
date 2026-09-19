import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { loader } from "@monaco-editor/react";
import * as monaco from "monaco-editor";
import App from "./App";
import SystemCenter from "./SystemCenter";
import KDevPowerPanel from "./KDevPowerPanel";
import FinishCenter from "./FinishCenter";
import "./styles.css";

import editorWorker from "monaco-editor/esm/vs/editor/editor.worker.js?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker.js?worker";
import cssWorker from "monaco-editor/esm/vs/language/css/css.worker.js?worker";
import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker.js?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker.js?worker";

(self as typeof self & {
  MonacoEnvironment?: {
    getWorker?: (workerId: string, label: string) => Worker | Promise<Worker>;
  };
}).MonacoEnvironment = {
  getWorker(_: string, label: string) {
    if (label === "json") return new jsonWorker();
    if (label === "css" || label === "scss" || label === "less") return new cssWorker();
    if (label === "html" || label === "handlebars" || label === "razor") return new htmlWorker();
    if (label === "typescript" || label === "javascript") return new tsWorker();
    return new editorWorker();
  }
};
loader.config({ monaco });
void loader.init().catch(() => undefined);

type SecurityState = { configured: boolean; config_path: string };

function LockScreen({ onUnlock }: { onUnlock: () => void }) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function unlock() {
    if (!password) return;
    setBusy(true);
    setError("");
    try {
      const ok = await invoke<boolean>("verify_master_password", { password });
      if (ok) onUnlock();
      else setError("Incorrect master password.");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return <div className="lock-screen">
    <div className="lock-card">
      <h2>KDev is locked</h2>
      <p>Enter your master password to open this workspace.</p>
      <input type="password" autoFocus value={password} onChange={e => setPassword(e.target.value)}
        onKeyDown={e => { if (e.key === "Enter") void unlock(); }} placeholder="Master password" />
      {error && <span className="lock-error">{error}</span>}
      <button disabled={busy || !password} onClick={() => void unlock()}>{busy ? "Checking…" : "Unlock"}</button>
    </div>
  </div>;
}

function Root() {
  const [powerOpen, setPowerOpen] = useState(false);
  const [finishOpen, setFinishOpen] = useState(false);
  const [securityConfigured, setSecurityConfigured] = useState<boolean | null>(null);
  const [unlocked, setUnlocked] = useState(false);

  useEffect(() => {
    invoke<SecurityState>("security_state")
      .then(s => setSecurityConfigured(s.configured))
      .catch(() => setSecurityConfigured(false));
  }, []);

  if (securityConfigured === null) return null;
  if (securityConfigured && !unlocked) return <LockScreen onUnlock={() => setUnlocked(true)} />;

  return <>
    <App />
    <SystemCenter />
    <button className="power-launch" onClick={() => setPowerOpen(true)} title="KDev Workspace Control">⌘</button>
    <button className="finish-launch" onClick={() => setFinishOpen(true)} title="KDev Development Control Center">✦</button>
    {powerOpen && <KDevPowerPanel onClose={() => setPowerOpen(false)} />}
    {finishOpen && <FinishCenter onClose={() => setFinishOpen(false)} />}
  </>;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode><Root /></React.StrictMode>
);
