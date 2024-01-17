import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App.tsx";
import "./global.css";

async function load() {
    ReactDOM.createRoot(document.getElementById("root")!).render(
        <React.StrictMode>
            <App />
        </React.StrictMode>,
    );
}
load().catch(console.error);
