import { mount } from "svelte";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "./app.css";
import App from "./App.svelte";
import PlcWindow from "./PlcWindow.svelte";
import { PLC } from "./lib/plcWindow";

// One bundle for both windows: the PLC window is told apart by its label (plan 2026-11-13-plc-window.md, decision 2).
const root = getCurrentWebviewWindow().label === PLC ? PlcWindow : App;
export default mount(root, { target: document.getElementById("app")! });
