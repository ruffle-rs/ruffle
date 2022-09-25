/**
 * Represents the various types of auto-play behaviours that are supported.
 */
export const enum AutoPlay {
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
export const enum Letterbox {
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
export const enum UnmuteOverlay {
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
export const enum LogLevel {
    Error = "error",
    Warn = "warn",
    Info = "info",
    Debug = "debug",
    Trace = "trace",
}

/**
 * The window mode of a Ruffle player.
 */
export const enum WindowMode {
    /*
     * The Flash content is rendered in its own window and layering is done with the browser's
     * default behavior.
     *
     * In Ruffle, this mode functions like `WindowMode::Opaque` and will layer the Flash content
     * together with other HTML elements.
     */
    Window = "window",

    /*
     * The Flash content is layered together with other HTML elements, and the stage color is
     * opaque. Content can render above or below Ruffle based on CSS rendering order.
     */
    Opaque = "opaque",

    /*
     * The Flash content is layered together with other HTML elements, and the SWF stage color is
     * transparent. Content beneath Ruffle will be visible through transparent areas.
     */
    Transparent = "transparent",

    /*
     * Request compositing with hardware acceleration when possible.
     * This mode has no effect in Ruffle and will function like `WindowMode.Opaque`.
     */
    Direct = "direct",

    /*
     * Request a direct rendering path, bypassing browser compositing when possible.
     * This mode has no effect in Ruffle and will function like `WindowMode::Opaque`.
     */
    Gpu = "gpu",
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
    parameters?: URLSearchParams | string | Record<string, string>;

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
     * Whether or not to display an overlay with a warning when
     * loading a movie with unsupported content.
     *
     * @default true
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
     * @default true
     */
    contextMenu?: boolean;

    /**
     * Whether or not to show a preloader before the SWF has loaded with Ruffle.
     *
     * @default true
     */
    preloader?: boolean;

    /**
     * Maximum amount of time a script can take before scripting
     * is disabled.
     *
     * @default { secs: 15, nanos: 0 }
     */
    maxExecutionDuration?: {
        secs: number;
        nanos: number;
    };

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
     * The window mode of the Ruffle player.
     *
     * This setting controls how the Ruffle container is layered and rendered with other content on the page.
     *
     * @default WindowMode.Window
     */
    wmode?: WindowMode;
}

/**
 * Options to load a movie by URL.
 */
export interface URLLoadOptions extends BaseLoadOptions {
    /**
     * The URL to load a movie from.
     *
     * If there is a query portion of this URL, then default [[parameters]]
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
    data: Iterable<number>;
}
