import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./system-center.css";

type SecurityState = { configured: boolean; config_path: string };
type GitStatus = { available: boolean; branch: string; changed: string[]; message: string };
type ProjectInfo = { has_package_json: boolean; has_requirements: boolean; has_pyproject: boolean; has_git: boolean; scripts: string[] };
type DebugCapability = { python: boolean; javascript: boolean; typescript: boolean; note: string };
type ExtensionInfo = { id: string; name: string; enabled: boolean };
type StorageInfo = { used: number; free: number; total: number; percent: number };

function bytes(n:number){ if(!Number.isFinite(n)) return "—"; const units=["B","KB","MB","GB","TB"]; let i=0,v=n; while(v>=1024&&i<units.length-1){v/=1024;i++;} return `${v.toFixed(v>=100?0:v>=10?1:2)} ${units[i]}`; }

export default function SystemCenter(){
 const [open,setOpen]=useState(false); const [tab,setTab]=useState("overview");
 const [security,setSecurity]=useState<SecurityState|null>(null); const [password,setPassword]=useState(""); const [confirm,setConfirm]=useState(""); const [locked,setLocked]=useState(false);
 const [project,setProject]=useState("Welcome"); const [projectInfo,setProjectInfo]=useState<ProjectInfo|null>(null); const [git,setGit]=useState<GitStatus|null>(null); const [debug,setDebug]=useState<DebugCapability|null>(null); const [extensions,setExtensions]=useState<ExtensionInfo[]>([]);
 const [storage,setStorage]=useState<StorageInfo|null>(null); const [message,setMessage]=useState(""); const [busy,setBusy]=useState(false);
 const projectReady=useMemo(()=>project.trim().length>0,[project]);
 async function refresh(){
   try{const s=await invoke<SecurityState>("security_state");setSecurity(s);}catch{}
   try{const g=await invoke<GitStatus>("git_status",{project:projectReady?project:undefined});setGit(g);}catch{}
   try{const p=await invoke<ProjectInfo>("project_info",{project});setProjectInfo(p);}catch{}
   try{setDebug(await invoke<DebugCapability>("debugger_capability"));}catch{}
   try{setExtensions(await invoke<ExtensionInfo[]>("list_extensions"));}catch{}
   try{const raw=await invoke<string>("storage_snapshot");const m=Object.fromEntries(raw.split("\n").map(x=>x.split("=")).filter(x=>x.length===2));const used=Number(m.workspace_used_bytes||0),free=Number(m.free_bytes||0),total=Number(m.filesystem_total_bytes||0);setStorage({used,free,total,percent:Number(m.filesystem_usage_percent||0)});}catch{}
 }
 useEffect(()=>{if(open)void refresh();},[open,project]);
 async function createPassword(){if(password.length<10){setMessage("Use at least 10 characters.");return;}if(password!==confirm){setMessage("Passwords do not match.");return;}setBusy(true);try{await invoke("set_master_password",{password});setMessage("Master password configured. KDev can now protect this environment.");setPassword("");setConfirm("");await refresh();}catch(e){setMessage(String(e));}finally{setBusy(false)}}
 async function unlock(){setBusy(true);try{const ok=await invoke<boolean>("verify_master_password",{password});if(ok){setLocked(false);setMessage("KDev unlocked.");setPassword("");}else setMessage("Incorrect master password.");}catch(e){setMessage(String(e));}finally{setBusy(false)}}
 async function install(){setBusy(true);try{const out=await invoke<string>("install_project_dependencies",{project});setMessage(out||"Dependency operation completed.");}catch(e){setMessage(String(e));}finally{setBusy(false);}}
 async function preview(){setBusy(true);try{const url=await invoke<string>("start_web_preview",{project,port:4173});setMessage(`Preview started at ${url}`);window.open(url,"_blank");}catch(e){setMessage(String(e));}finally{setBusy(false);}}
 if(!open)return <button className="system-fab" onClick={()=>setOpen(true)} title="KDev System Center">⚙</button>;
 return <div className="system-overlay"><section className="system-window">
   <header><div><span className="system-kicker">KDEV SYSTEMS</span><h2>System Center</h2></div><button onClick={()=>setOpen(false)}>×</button></header>
   <div className="system-body"><aside>{[["overview","Overview"],["security","Security"],["project","Project"],["git","Git"],["debug","Debugger"],["extensions","Extensions"],["storage","Storage"]].map(([id,label])=><button key={id} className={tab===id?"active":""} onClick={()=>setTab(id)}>{label}</button>)}</aside>
   <main>
    {tab==="overview"&&<><div className="system-hero"><span>PORTABLE DEVELOPMENT OS</span><h3>Everything KDev needs, in one place.</h3><p>Local-first workspace controls, diagnostics, security, Git, debugging, extensions and storage health.</p></div><div className="system-grid"><div className="system-card"><b>Workspace</b><strong>{project}</strong><small>{projectInfo?.has_git?"Git enabled":"Local project"}</small></div><div className="system-card"><b>Security</b><strong>{security?.configured?"Protected":"Not configured"}</strong><small>Master-password vault</small></div><div className="system-card"><b>Debugger</b><strong>{debug?.python?"Python ready":"Detecting"}</strong><small>JS/TS: {debug?.javascript?"available":"not detected"}</small></div><div className="system-card"><b>Storage</b><strong>{storage?bytes(storage.free):"Checking…"}</strong><small>free on KDev filesystem</small></div></div></>}
    {tab==="security"&&<div className="system-section"><h3>Workspace Security</h3><p className="muted">Your password is never stored as plaintext. KDev stores a password verifier and uses it to protect the local environment.</p>{security?.configured&&!locked?<div className="security-ok">✓ Master password configured</div>:security?.configured?<><input type="password" value={password} onChange={e=>setPassword(e.target.value)} placeholder="Master password"/><button className="primary" disabled={busy} onClick={()=>void unlock()}>Unlock KDev</button></>:<><input type="password" value={password} onChange={e=>setPassword(e.target.value)} placeholder="Create master password (10+ chars)"/><input type="password" value={confirm} onChange={e=>setConfirm(e.target.value)} placeholder="Confirm password"/><button className="primary" disabled={busy} onClick={()=>void createPassword()}>Create Master Password</button></>}<p className="notice">KDev will not lock you out of source code. Exported projects remain ordinary files.</p></div>}
    {tab==="project"&&<div className="system-section"><h3>Project Services</h3><input value={project} onChange={e=>setProject(e.target.value)} placeholder="Project name"/><div className="pill-row">{projectInfo&&<><span>{projectInfo.has_package_json?"npm":"no package.json"}</span><span>{projectInfo.has_requirements||projectInfo.has_pyproject?"Python deps":"no Python manifest"}</span><span>{projectInfo.has_git?"Git repo":"not a Git repo"}</span></>}</div><div className="button-row"><button disabled={busy} onClick={()=>void install()}>Install Dependencies</button><button disabled={busy||!projectInfo?.has_package_json&&!projectInfo?.has_git} onClick={()=>void preview()}>Web Preview</button></div>{projectInfo?.scripts.length?<><h4>Scripts</h4><div className="script-list">{projectInfo.scripts.map(s=><span key={s}>npm run {s}</span>)}</div></>:null}</div>}
    {tab==="git"&&<div className="system-section"><h3>Source Control</h3>{git?<><div className={git.available?"security-ok":"notice"}>{git.available?`✓ ${git.branch}`:git.message}</div><h4>Working tree</h4>{git.changed.length?<div className="change-list">{git.changed.map((x,i)=><code key={i}>{x}</code>)}</div>:<p className="muted">No working-tree changes detected.</p>}</>:<p>Checking Git…</p>}</div>}
    {tab==="debug"&&<div className="system-section"><h3>Debugger</h3><div className="system-grid"><div className="system-card"><b>Python</b><strong>{debug?.python?"READY":"MISSING"}</strong><small>debugpy capability</small></div><div className="system-card"><b>JavaScript</b><strong>{debug?.javascript?"READY":"MISSING"}</strong><small>Node runtime</small></div><div className="system-card"><b>TypeScript</b><strong>{debug?.typescript?"READY":"MISSING"}</strong><small>Node-based adapter</small></div></div><p className="muted">{debug?.note||"Detecting local debugger adapters…"}</p></div>}
    {tab==="extensions"&&<div className="system-section"><h3>Extensions</h3><p className="muted">Extensions are isolated packages. KDev does not silently grant them access to projects or external commands.</p>{extensions.length?<div className="extension-list">{extensions.map(x=><div key={x.id}><b>{x.name}</b><span>{x.enabled?"Enabled":"Disabled"}</span></div>)}</div>:<div className="notice">No extensions installed yet. The extension directory is ready.</div>}</div>}
    {tab==="storage"&&<div className="system-section"><h3>Portable Storage</h3>{storage?<><div className="storage-meter"><div style={{width:`${Math.min(100,Math.max(0,storage.percent))}%`}}/></div><div className="storage-stats"><span><b>{bytes(storage.free)}</b> free</span><span><b>{bytes(storage.total)}</b> total</span><span><b>{storage.percent.toFixed(1)}%</b> workspace usage</span></div><p className="muted">Workspace data stays beside KDev so the drive can move between machines without hard-coded paths.</p></>:<p>Checking storage…</p>}</div>}
    {message&&<div className="system-message">{message}</div>}
   </main></div>
 </section></div>;
}
