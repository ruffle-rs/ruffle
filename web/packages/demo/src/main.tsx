import React from "react";
import ReactDOM from "react-dom/client";
import "./common.css";
import "./lato.css";
import "./index.css";
import { App } from "./App.tsx";
import { Config, Setup } from "ruffle-core";

Setup.installRuffle("local");

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <App
            ruffleBaseConfig={{
                autoplay: Config.AutoPlay.On,
                unmuteOverlay: Config.UnmuteOverlay.Hidden,
                logLevel: Config.LogLevel.Warn,
                letterbox: Config.Letterbox.On,
                forceScale: true,
                forceAlign: true,
            }}
            allowSampleSwfs={true}
            allowUrlLoading={false}
        />
    </React.StrictMode>,
);
