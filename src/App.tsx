import { useEffect, useMemo, useState } from "react";
import Editor from "@monaco-editor/react";
import { invoke } from "@tauri-apps/api/core";
import linuxWallpaper from "./assets/kdev-linux-wallpaper.svg";

type FileItem = { path: string; name: string; language: string };
type PlatformInfo = { os: string; arch: string; kdevRoot: string };

type Workspace = "dev" | "linux";

const starterFiles: FileItem[] = [
  { path: "workspace/projects/Welcome/main.py", name: "main.py", language: "python" },
  { path: "workspace/projects/Welcome/index.html", name: "index.html", language: "html" },
  { path: "workspace/projects/Welcome/app.js", name: "app.js", language: "javascript" },
  { path: "workspace/projects/Welcome/style.css", name: "style.css", language: "css" },
];

const starterContent: Record<string, string> = {
  "workspace/projects/Welcome/main.py": "print(\"Welcome to KDev\")\n",
  "workspace/projects/Welcome/index.html": "<!doctype html>\n<html>\n  <body>\n    <h1>KDev</h1>\n  </body>\n</html>\n",
  "workspace/projects/Welcome/app.js": "const message = \"KDev is running\";\nconsole.log(message);\n",
  "workspace/projects/Welcome/style.css": "body {\n  font-family: system-ui, sans-serif;\n}\n",
};

function languageFor(path: string) {
  const ext = path.split(".").pop()?.toLowerCase();
  return ({ py: "python", js: "javascript", jsx: "javascript", ts: "typescript", tsx: "typescript", html: "html", css: "css", json: "json", md: "markdown", sh: "shell" } as Record<string, string>)[ext ?? ""] ?? "plaintext";
}

function App() {
  const [files] = useState<FileItem[]>(starterFiles);
  const [activePath, setActivePath] = useState(starterFiles[0].path);
  const [contents, setContents] = useState<Record<string, string>>(starterContent);
  const [platform, setPlatform] = useState<PlatformInfo | null>(null);
  const [saved, setSaved] = useState(true);
  const [workspace, setWorkspace] = useState<Workspace>("dev");
  const [terminal, setTerminal] = useState("KDev terminal ready. Offline-first workspace initialized.\n");
  const [linuxCommand, setLinuxCommand] = useState("");

  const activeFile = useMemo(() => files.find((file) => file.path === activePath) ?? files[0], [activePath, files]);

  useEffect(() => {
    invoke<PlatformInfo>("platform_info").then(setPlatform).catch(() => {
      setPlatform({ os: navigator.platform, arch: "browser", kdevRoot: "development mode" });
    });
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      if (!saved) void saveCurrentFile();
    }, 700);
    return () => window.clearTimeout(timer);
  }, [contents, activePath, saved]);

  async function saveCurrentFile() {
    const content = contents[activePath] ?? "";
    try {
      await invoke("write_workspace_file", { relativePath: activePath, content });
      setSaved(true);
      setTerminal((value) => `${value}> saved ${activePath}\n`);
    } catch {
      localStorage.setItem(`kdev:${activePath}`, content);
      setSaved(true);
      setTerminal((value) => `${value}> local recovery save: ${activePath}\n`);
    }
  }

  function updateContent(value: string | undefined) {
    setContents((current) => ({ ...current, [activePath]: value ?? "" }));
    setSaved(false);
  }

  function runLinuxCommand() {
    const command = linuxCommand.trim();
    if (!command) return;
    setTerminal((value) => `${value}\nlinux@kdev:~$ ${command}\n`);
    setLinuxCommand("");
  }

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="brand"><span className="brand-mark">K</span><strong>KDev</strong></div>
        <nav className="menu"><button>File</button><button>Edit</button><button>View</button><button>Terminal</button><button>Run</button><button>Tools</button><button>Help</button></nav>
        <div className="workspace-switcher">
          <button className={workspace === "dev" ? "selected" : ""} onClick={() => setWorkspace("dev")}>DEV</button>
          <button className={workspace === "linux" ? "selected linux" : ""} onClick={() => setWorkspace("linux")}>🐧 LINUX</button>
        </div>
        <div className="connection"><span className="online-dot" /> {navigator.onLine ? "ONLINE" : "OFFLINE"}</div>
      </header>

      {workspace === "linux" ? (
        <main className="linux-workspace" style={{ backgroundImage: `linear-gradient(rgba(5,6,8,.18), rgba(5,6,8,.55)), url(${linuxWallpaper})` }}>
          <div className="linux-topbar"><strong>🐧 KDev Linux</strong><span>Debian-based workspace</span><span className="linux-spacer" /><span>OFFLINE LAB</span></div>
          <div className="linux-desktop">
            <div className="linux-card hero-card">
              <div className="eyebrow">LINUX WORKSPACE</div>
              <h1>Feel Linux. Build freely.</h1>
              <p>A dedicated Linux-style workspace for learning, development and controlled security labs.</p>
              <div className="linux-chips"><span>BASH</span><span>GIT</span><span>PYTHON</span><span>SSH</span><span>NETWORKING</span></div>
            </div>
            <div className="linux-card terminal-card">
              <div className="linux-card-title"><span>TERMINAL</span><span>bash</span></div>
              <pre>{terminal}</pre>
              <div className="linux-prompt"><span>linux@kdev:~$</span><input value={linuxCommand} onChange={(event) => setLinuxCommand(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") runLinuxCommand(); }} placeholder="try: pwd" autoFocus /></div>
            </div>
            <div className="linux-card tool-card">
              <div className="linux-card-title"><span>TOOLBOX</span><span>LOCAL</span></div>
              <button onClick={() => setTerminal((value) => `${value}\n$ whoami\nkdev-user\n`)}>whoami</button>
              <button onClick={() => setTerminal((value) => `${value}\n$ pwd\n~/KDev/linux-workspace\n`)}>pwd</button>
              <button onClick={() => setTerminal((value) => `${value}\n$ git status\nOn branch main\nworking tree clean\n`)}>git status</button>
              <button onClick={() => setTerminal((value) => `${value}\nKDev Security Learning\nUse security tooling only on systems and labs you are authorized to test.\n`)}>Security Learning</button>
            </div>
          </div>
          <div className="linux-dock"><button onClick={() => setWorkspace("dev")}>▣ Developer Workspace</button><button>⌘ Files</button><button>⌁ Network Lab</button><button>◈ Documentation</button><button>⚙ Settings</button></div>
        </main>
      ) : (
        <main className="workbench">
          <aside className="explorer panel">
            <div className="panel-title"><span>EXPLORER</span><span className="muted">WORKSPACE</span></div>
            <div className="tree-root">KDev</div><div className="tree-folder">workspace</div><div className="tree-folder indent">projects</div><div className="tree-folder indent-2">Welcome</div>
            {files.map((file) => <button key={file.path} className={`tree-file ${file.path === activePath ? "active" : ""}`} onClick={() => setActivePath(file.path)}><span className="file-icon">{file.name.endsWith(".py") ? "◆" : file.name.endsWith(".css") ? "#" : file.name.endsWith(".html") ? "<>" : "JS"}</span>{file.name}</button>)}
          </aside>
          <section className="editor-area">
            <div className="tabs"><div className="tab active"><span>{activeFile.name}</span><span className="tab-dot">{saved ? "" : "●"}</span><button onClick={saveCurrentFile}>×</button></div></div>
            <div className="editor-wrap"><Editor theme="vs-dark" language={languageFor(activePath)} value={contents[activePath] ?? ""} onChange={updateContent} options={{ automaticLayout: true, minimap: { enabled: true }, fontSize: 14, fontLigatures: true, smoothScrolling: true, tabSize: 2, suggestOnTriggerCharacters: true, quickSuggestions: true }} /></div>
            <div className="terminal"><div className="terminal-tabs"><span className="selected">TERMINAL</span><span>OUTPUT</span><span>PROBLEMS</span><button onClick={() => setTerminal("")}>Clear</button></div><pre>{terminal}</pre></div>
          </section>
          <aside className="side-panel panel"><div className="panel-title"><span>OUTLINE</span></div><div className="outline-item">{activeFile.name}</div><div className="outline-item muted">Symbols will appear here as language intelligence is connected.</div><div className="panel-title lower"><span>PROJECT</span></div><div className="project-card"><strong>Welcome</strong><span>Portable workspace</span><span>Autosave: {saved ? "saved" : "saving…"}</span></div><div className="panel-title lower"><span>KDEV</span></div><div className="about">Built & developed by <strong>KAGIMU DALTON</strong><br /><span>Portable • Offline-first • Developer focused</span></div></aside>
        </main>
      )}

      <footer className="statusbar"><span>✓ {saved ? "Saved locally" : "Unsaved changes"}</span><span>{activeFile.language}</span><span>{platform ? `${platform.os} • ${platform.arch}` : "Detecting platform…"}</span><span className="status-spacer" /><span>{workspace === "linux" ? "LINUX WORKSPACE" : "UTF-8"}</span><span>{workspace === "dev" ? "Spaces: 2" : "Bash"}</span><span>{workspace === "dev" ? "Ln 1, Col 1" : "LOCAL"}</span></footer>
    </div>
  );
}

export default App;
