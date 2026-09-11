import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import SystemCenter from "./SystemCenter";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
    <SystemCenter />
  </React.StrictMode>
);
