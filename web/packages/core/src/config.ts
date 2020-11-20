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
 * The configuration object to control Ruffle's behaviour on the website
 * that it is included on.
 */
export interface Config {
    /**
     * A map of public paths from source name to URL.
     */
    publicPaths?: Record<string, string>;

    /**
     * The URL at which Ruffle can load its extra files (ie `.wasm`).
     *
     * [publicPaths] is consulted first for a source-specific URL,
     * with this field being a fallback.
     */
    publicPath?: string;

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
     * Controls the auto-play behaviour of Ruffle.
     *
     * @default AutoPlay.Auto
     */
    autoplay?: AutoPlay;

    /**
     * Controls the visiblity of the unmute overlay when the player
     * is started muted.
     *
     * @default UnmuteOverlay.Visible
     */
    unmuteOverlay?: UnmuteOverlay;
}
