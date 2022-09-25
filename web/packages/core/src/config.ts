import type { BaseLoadOptions } from "./load-options";
import {
    AutoPlay,
    UnmuteOverlay,
    WindowMode,
    Letterbox,
    LogLevel,
} from "./load-options";

/**
 * The configuration object to control Ruffle's behaviour on the website
 * that it is included on.
 */
export interface Config extends BaseLoadOptions {
    /**
     * The URL at which Ruffle can load its extra files (i.e. `.wasm`).
     *
     * @default null
     */
    publicPath?: string | null;

    /**
     * Whether or not to enable polyfills on the page.
     *
     * Polyfills will look for "legacy" flash content like `<object>`
     * and `<embed>` elements, and replace them with compatible
     * Ruffle elements.
     *
     * @default true
     */
    polyfills?: boolean;
}

export const DEFAULT_CONFIG: Required<Config> = {
    allowScriptAccess: false,
    parameters: {},
    autoplay: AutoPlay.Auto,
    backgroundColor: null,
    letterbox: Letterbox.Fullscreen,
    unmuteOverlay: UnmuteOverlay.Visible,
    upgradeToHttps: true,
    warnOnUnsupportedContent: true,
    logLevel: LogLevel.Error,
    showSwfDownload: false,
    contextMenu: true,
    preloader: true,
    maxExecutionDuration: { secs: 15, nanos: 0 },
    base: null,
    menu: true,
    salign: "",
    quality: "high",
    scale: "showAll",
    wmode: WindowMode.Opaque,
    publicPath: null,
    polyfills: true,
};
