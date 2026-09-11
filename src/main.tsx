import React, { useState } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import SystemCenter from "./SystemCenter";
import KDevPowerPanel from "./KDevPowerPanel";
import FinishCenter from "./FinishCenter";
import "./styles.css";

function Root() {
  const [powerOpen, setPowerOpen] = useState(false);
  const [finishOpen, setFinishOpen] = useState(false);
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
