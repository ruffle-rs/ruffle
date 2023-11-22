import React from "react";
import ReactDOM from "react-dom/client";
import "./common.css";
import "./lato.css";
import "./index.css";
import { App } from "./App.tsx";
import {
    AutoPlay,
    Letterbox,
    LogLevel,
    PublicAPI,
    UnmuteOverlay,
} from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(window.RufflePlayer, "local");

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <App
            ruffleBaseConfig={{
                autoplay: AutoPlay.On,
                unmuteOverlay: UnmuteOverlay.Hidden,
                logLevel: LogLevel.Warn,
                letterbox: Letterbox.On,
                forceScale: true,
                forceAlign: true,
            }}
            allowSampleSwfs={true}
            allowUrlLoading={false}
        />
    </React.StrictMode>,
);
