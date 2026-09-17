import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import SystemCenter from "./SystemCenter";
import KDevPowerPanel from "./KDevPowerPanel";
import FinishCenter from "./FinishCenter";
import "./styles.css";

type SecurityState = { configured: boolean; config_path: string };

/// Gates the whole app behind the master password when one has been
/// configured. Unlocking is per-session (not persisted to disk): closing
/// and reopening KDev with a configured master password always asks again.
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
      <input
        type="password"
        autoFocus
        value={password}
        onChange={e => setPassword(e.target.value)}
        onKeyDown={e => { if (e.key === "Enter") void unlock(); }}
        placeholder="Master password"
      />
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

  // Nothing configured yet, or still checking: don't block the workspace.
  if (securityConfigured === null) return null;
  if (securityConfigured && !unlocked) {
    return <LockScreen onUnlock={() => setUnlocked(true)} />;
  }

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
