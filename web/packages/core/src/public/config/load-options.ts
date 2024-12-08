/**
 * Represents the various types of auto-play behaviours that are supported.
 */
export enum AutoPlay {
    /**
     * The player should automatically play the movie as soon as it is loaded.
     *
     * If the browser does not support automatic audio, the movie will begin
     * muted.
     */
    On = "on",

    /**
     * The player should not attempt to automatically play the movie.
     *
     * This will leave it to the user or API to actually play when appropriate.
     */
    Off = "off",

    /**
     * The player should automatically play the movie as soon as it is deemed
     * "appropriate" to do so.
     *
     * The exact behaviour depends on the browser, but commonly requires some
     * form of user interaction on the page in order to allow auto playing videos
     * with sound.
     */
    Auto = "auto",
}

/**
 * Controls whether the content is letterboxed or pillarboxed when the
 * player's aspect ratio does not match the movie's aspect ratio.
 *
 * When letterboxed, black bars will be rendered around the exterior
 * margins of the content.
 */
export enum Letterbox {
    /**
     * The content will never be letterboxed.
     */
    Off = "off",

    /**
     * The content will only be letterboxed if the content is running fullscreen.
     */
    Fullscreen = "fullscreen",

    /**
     * The content will always be letterboxed.
     */
    On = "on",
}

/**
 * When the player is muted, this controls whether or not Ruffle will show a
 * "click to unmute" overlay on top of the movie.
 */
export enum UnmuteOverlay {
    /**
     * Show an overlay explaining that the movie is muted.
     */
    Visible = "visible",

    /**
     * Don't show an overlay and pretend that everything is fine.
     */
    Hidden = "hidden",
}

/**
 * Console logging level.
 */
export enum LogLevel {
    Error = "error",
    Warn = "warn",
    Info = "info",
    Debug = "debug",
    Trace = "trace",
}

/**
 * The window mode of a Ruffle player.
 */
export enum WindowMode {
    /**
     * The Flash content is rendered in its own window and layering is done with the browser's
     * default behavior.
     *
     * In Ruffle, this mode functions like `WindowMode::Opaque` and will layer the Flash content
     * together with other HTML elements.
     */
    Window = "window",

    /**
     * The Flash content is layered together with other HTML elements, and the stage color is
     * opaque. Content can render above or below Ruffle based on CSS rendering order.
     */
    Opaque = "opaque",

    /**
     * The Flash content is layered together with other HTML elements, and the SWF stage color is
     * transparent. Content beneath Ruffle will be visible through transparent areas.
     */
    Transparent = "transparent",

    /**
     * Request compositing with hardware acceleration when possible.
     * This mode has no effect in Ruffle and will function like `WindowMode.Opaque`.
     */
    Direct = "direct",

    /**
     * Request a direct rendering path, bypassing browser compositing when possible.
     * This mode has no effect in Ruffle and will function like `WindowMode::Opaque`.
     */
    Gpu = "gpu",
}

/**
 * The render backend of a Ruffle player.
 *
 * The available backends may change in future releases.
 */
export enum RenderBackend {
    /**
     * An [in-development API](https://caniuse.com/webgpu) that will be preferred if available in the future.
     * Should behave the same as wgpu-webgl, except with lower overhead and thus better performance.
     */
    WebGpu = "webgpu",

    /**
     * The most featureful and currently preferred backend.
     * Rendering is done the same way as in the desktop app, then translated to WebGL on-the-fly.
     */
    WgpuWebgl = "wgpu-webgl",

    /**
     * A vanilla WebGL backend. Was the default backend until the start of 2023,
     * but is now used as a fallback for devices that do not support WebGL 2.
     * Supports fewer features and has a faster initialization time;
     * may be useful for content that does not need advanced features like bitmap drawing or blend modes.
     */
    Webgl = "webgl",

    /**
     * The slowest and most basic backend, used as a fallback when all else fails.
     * However, this is currently the only backend that accurately scales hairline strokes.
     * If you notice excessively thick strokes in specific content,
     * you may want to use the canvas renderer for that content until the issue is resolved.
     */
    Canvas = "canvas",
}

/**
 * Represents the various context menu options that are supported.
 */
export enum ContextMenu {
    /**
     * The context menu should appear when right-clicking or long-pressing
     * the Ruffle instance.
     */
    On = "on",

    /**
     * The context menu should only appear when right-clicking
     * the Ruffle instance.
     */
    RightClickOnly = "rightClickOnly",

    /**
     * The context menu should not appear when right-clicking or long-pressing
     * the Ruffle instance.
     */
    Off = "off",
}

/**
 * Represents the player runtime to emulate.
 */
export enum PlayerRuntime {
    /**
     * Emulate Adobe AIR.
     */
    AIR = "air",

    /**
     * Emulate Adobe Flash Player.
     */
    FlashPlayer = "flashPlayer",
}

/**
 * Non-negative duration in seconds.
 */
export type SecsDuration = number;

/**
 * Deprecated duration type, use SecsDuration instead.
 * Based on https://doc.rust-lang.org/stable/std/time/struct.Duration.html#method.new .
 */
export interface ObsoleteDuration {
    secs: number;
    nanos: number;
}

/**
 * Any new duration-based setting should use 'number' or 'SecsDuration' for its type,
 * instead of this type.
 */
export type Duration = SecsDuration | ObsoleteDuration;

/**
 * The handling mode of links opening a new website.
 */
export enum OpenURLMode {
    /**
     * Allow all links to open a new website.
     */
    Allow = "allow",

    /**
     * A confirmation dialog opens with every link trying to open a new website.
     */
    Confirm = "confirm",

    /**
     * Deny all links to open a new website.
     */
    Deny = "deny",
}

/**
 * The networking API access mode of the Ruffle player.
 */
export enum NetworkingAccessMode {
    /**
     * All networking APIs are permitted in the SWF file.
     */
    All = "all",

    /**
     * The SWF file may not call browser navigation or browser interaction APIs.
     *
     * The APIs navigateToURL(), fscommand() and ExternalInterface.call() are
     * prevented in this mode.
     */
    Internal = "internal",

    /**
     * The SWF file may not call browser navigation or browser interaction APIs
     * and it cannot use any SWF-to-SWF communication APIs.
     *
     * Additionally to the ones in internal mode, the APIs sendToURL(),
     * FileReference.download(), FileReference.upload(), Loader.load(),
     * LocalConnection.connect(), LocalConnection.send(), NetConnection.connect(),
     * NetStream.play(), Security.loadPolicyFile(), SharedObject.getLocal(),
     * SharedObject.getRemote(), Socket.connect(), Sound.load(), URLLoader.load(),
     * URLStream.load() and XMLSocket.connect() are prevented in this mode.
     *
     * This mode is not implemented yet.
     */
    None = "none",
}

/**
 * Represents a host, port and proxyUrl. Used when a SWF file tries to use a Socket.
 */
export interface SocketProxy {
    /**
     * Host used by the SWF.
     */
    host: string;
    /**
     * Port used by the SWF.
     */
    port: number;

    /**
     * The proxy URL to use when SWF file tries to connect to the specified host and port.
     */
    proxyUrl: string;
}

/**
 * Defines the names of the fonts to use for each "default" Flash device font.
 *
 * The name of each font provided will be used, in priority order.
 *
 * For example, defining `sans: ["Helvetica", "Arial"]` would use Helvetica if present, before trying Arial.
 */
export interface DefaultFonts {
    /**
     * `_sans`, a Sans-Serif font (similar to Helvetica or Arial)
     */
    sans?: Array<string>;

    /**
     * `_serif`, a Serif font (similar to Times Roman)
     */
    serif?: Array<string>;

    /**
     * `_typewriter`, a Monospace font (similar to Courier)
     */
    typewriter?: Array<string>;

    /**
     * `_ゴシック`, a Japanese Gothic font
     */
    japaneseGothic?: Array<string>;

    /**
     * `_等幅`, a Japanese Gothic Mono font
     */
    japaneseGothicMono?: Array<string>;

    /**
     * `_明朝`, a Japanese Mincho font
     */
    japaneseMincho?: Array<string>;
}

/**
 * Any options used for loading a movie.
 */
export interface BaseLoadOptions {
    /**
     * If set to true, the movie is allowed to interact with the page through
     * JavaScript, using a flash concept called `ExternalInterface`.
     *
     * This should only be enabled for movies you trust.
     *
     * @default false
     */
    allowScriptAccess?: boolean;

    /**
     * Also known as "flashvars" - these are values that may be passed to
     * and loaded by the movie.
     *
     * If a URL if specified when loading the movie, some parameters will
     * be extracted by the query portion of that URL and then overwritten
     * by any explicitly set here.
     *
     * @default {}
     */
    parameters?: URLSearchParams | string | Record<string, string> | null;

    /**
     * Controls the auto-play behaviour of Ruffle.
     *
     * @default AutoPlay.Auto
     */
    autoplay?: AutoPlay;

    /**
     * Controls the background color of the player.
     * Must be an HTML color (e.g. "#FFFFFF"). CSS colors are not allowed.
     * `null` uses the background color of the SWF file.
     *
     * @default null
     */
    backgroundColor?: string | null;

    /**
     * Controls letterbox behavior when the Flash container size does not
     * match the movie size.
     *
     * @default Letterbox.Fullscreen
     */
    letterbox?: Letterbox;

    /**
     * Controls the visibility of the unmute overlay when the player
     * is started muted.
     *
     * @default UnmuteOverlay.Visible
     */
    unmuteOverlay?: UnmuteOverlay;

    /**
     * Whether or not to auto-upgrade all embedded URLs to https.
     *
     * Flash content that embeds http urls will be blocked from
     * accessing those urls by the browser when Ruffle is loaded
     * in a https context. Set to `true` to automatically change
     * `http://` to `https://` for all embedded URLs when Ruffle is
     * loaded in an https context.
     *
     * @default true
     */
    upgradeToHttps?: boolean;

    /**
     * Enable (true) or disable (false) Ruffle's built in compatibility rules.
     *
     * These are rules that may make some content work by deliberately changing
     * behaviour, for example by rewriting requests or spoofing SWF urls if they
     * rely on websites that no longer exist.
     *
     * @default true
     */
    compatibilityRules?: boolean;

    /**
     * Favor using the real Adobe Flash Player over Ruffle if the browser supports it.
     *
     * @default true
     */
    favorFlash?: boolean;

    /**
     * This is no longer used and does not affect anything.
     * It is only kept for backwards compatibility.
     *
     * Previously:
     * "Whether or not to display an overlay with a warning when
     * loading a movie with unsupported content."
     *
     * @default true
     * @deprecated
     */
    warnOnUnsupportedContent?: boolean;

    /**
     * Console logging level.
     *
     * @default LogLevel.Error
     */
    logLevel?: LogLevel;

    /**
     * If set to true, the context menu has an option to download
     * the SWF.
     *
     * @default false
     */
    showSwfDownload?: boolean;

    /**
     * Whether or not to show a context menu when right-clicking
     * a Ruffle instance.
     *
     * @default ContextMenu.On
     */
    contextMenu?: ContextMenu | boolean;

    /**
     * Whether or not to show a splash screen before the SWF has loaded with Ruffle (backwards-compatibility).
     *
     * @default true
     */
    preloader?: boolean;

    /**
     * Whether or not to show a splash screen before the SWF has loaded with Ruffle.
     *
     * @default true
     */
    splashScreen?: boolean;

    /**
     * Maximum amount of time a script can take before scripting
     * is disabled.
     *
     * @default 15
     */
    maxExecutionDuration?: Duration;

    /**
     * Specifies the base directory or URL used to resolve all relative path statements in the SWF file.
     * null means the current directory.
     *
     * @default null
     */
    base?: string | null;

    /**
     * If set to true, the built-in context menu items are visible
     *
     * This is equivalent to Stage.showMenu.
     *
     * @default true
     */
    menu?: boolean;

    /**
     * This is equivalent to Stage.align.
     *
     * @default ""
     */
    salign?: string;

    /**
     * If set to true, movies are prevented from changing the stage alignment.
     *
     * @default false
     */
    forceAlign?: boolean;

    /**
     * This is equivalent to Stage.quality.
     *
     * @default "high"
     */
    quality?: string;

    /**
     * This is equivalent to Stage.scaleMode.
     *
     * @default "showAll"
     */
    scale?: string;

    /**
     * If set to true, movies are prevented from changing the stage scale mode.
     *
     * @default false
     */
    forceScale?: boolean;

    /**
     * If set to true, the Stage's displayState can be changed
     *
     * @default false
     */
    allowFullscreen?: boolean;

    /**
     * Sets and locks the player's frame rate, overriding the movie's frame rate.
     *
     * @default null
     */
    frameRate?: number | null;

    /**
     * The window mode of the Ruffle player.
     *
     * This setting controls how the Ruffle container is layered and rendered with other content on the page.
     *
     * @default WindowMode.Window
     */
    wmode?: WindowMode;

    /**
     * The emulated version of the player.
     *
     * This controls the version that is reported to the movie.
     * null means latest version.
     *
     * @default null
     */
    playerVersion?: number | null;

    /**
     * The preferred render backend of the Ruffle player.
     *
     * This option should only be used for testing;
     * the available backends may change in future releases.
     * By default, Ruffle chooses the most featureful backend supported by the user's system,
     * falling back to more basic backends if necessary.
     * The available values in order of default preference are:
     * "webgpu", "wgpu-webgl", "webgl", "canvas".
     *
     * @default null
     */
    preferredRenderer?: RenderBackend | null;

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

    /**
     * The handling mode of links opening a new website.
     *
     * @default OpenURLMode.Allow
     */
    openUrlMode?: OpenURLMode;

    /**
     * Which flash networking APIs may be accessed.
     *
     * @default NetworkingAccessMode.All
     */
    allowNetworking?: NetworkingAccessMode;

    /**
     * A function to call for opening content in a new tab.
     *
     * This is only used if the content cannot be loaded due to CORS,
     * and the Extension version of Ruffle will override this to provide a local player.
     *
     * @default null
     */
    openInNewTab?: ((swf: URL) => void) | null;

    /**
     * An array of SocketProxy objects.
     *
     * When a SWF tries to establish a Socket connection, Ruffle will search for
     * a matching SocketProxy object in this array and use it to establish a WebSocket connection,
     * through which all communication is tunneled through.
     *
     * When none are found, Ruffle will fail the connection gracefully.
     * When multiple matching SocketProxy objects exist, the first one is used.
     *
     * @default []
     */
    socketProxy?: Array<SocketProxy>;

    /**
     * An array of font URLs to eagerly load and provide to Ruffle.
     *
     * These will be fetched by the browser as part of the loading of Flash content, which may slow down load times.
     *
     * Currently only SWFs are supported, and each font embedded within that SWF will be used as device font by Flash content.
     *
     * If any URL fails to load (either it's an invalid file, or a network error occurs), Ruffle will log an error but continue without it.
     *
     * @default []
     */
    fontSources?: Array<string>;

    /**
     * The font names to use for each "default" Flash device font.
     *
     * @default {}
     */
    defaultFonts?: DefaultFonts;

    /**
     * An array of origins that credentials may be sent to.
     * Credentials are cookies, authorization headers, or TLS client certificates.
     *
     * Entries should include the protocol and host, for example `https://example.org` or `http://subdomain.example.org`.
     *
     * Cookies will always be sent to the same origin as the page the content was loaded on.
     * If you configure this to send cookies to an origin but that origin does not configure CORS to allow it,
     * then requests will start failing due to CORS.
     * See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials.
     *
     * This directly corresponds to https://developer.mozilla.org/en-US/docs/Web/API/fetch#credentials
     * Every request will be `same-origin` unless specified here, in which case it will be `include`.
     *
     * @default []
     */
    credentialAllowList?: Array<string>;

    /**
     * The player runtime to emulate
     *
     * This allows you to emulate Adobe AIR or Adobe Flash Player.
     */
    playerRuntime?: PlayerRuntime;

    /**
     * A set of rules that rewrite URLs in both network requests and links.
     *
     * They are always scanned in order, and the first one that matches is used.
     * A rule either matches using a RegExp (in which case the replacement may use `$...`),
     * or a string (in which case the match and the replacement are always exact).
     *
     * They are useful when a SWF uses an obsolete URL, in which case
     * you can rewrite it to something else that works.
     */
    urlRewriteRules?: Array<[RegExp | string, string]>;
}

/**
 * Options to load a movie by URL.
 */
export interface URLLoadOptions extends BaseLoadOptions {
    /**
     * The URL to load a movie from.
     *
     * If there is a query portion of this URL, then default {@link parameters}
     * will be extracted from that.
     */
    url: string;
}

/**
 * Options to load a movie by a data stream.
 */
export interface DataLoadOptions extends BaseLoadOptions {
    /**
     * The data to load a movie from.
     */
    data: ArrayLike<number> | ArrayBufferLike;

    /**
     * The filename of the SWF movie to provide to ActionScript.
     *
     * @default "movie.swf"
     */
    swfFileName?: string;
}
