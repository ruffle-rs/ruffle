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
 * Any options used for loading a movie.
 */
export interface BaseLoadOptions {
    /**
     * Also known as "flashvars" - these are values that may be passed to
     * and loaded by the movie.
     *
     * If a URL if specified when loading the movie, some parameters will
     * be extracted by the query portion of that URL and then overwritten
     * by any explicitly set here.
     */
    parameters?: URLSearchParams | string | Record<string, string>;

    /**
     * Controls the auto-play behaviour of Ruffle.
     *
     * @default AutoPlay.Auto
     */
    autoplay?: AutoPlay;

    /**
     * Controls the visibility of the unmute overlay when the player
     * is started muted.
     *
     * @default UnmuteOverlay.Visible
     */
    unmuteOverlay?: UnmuteOverlay;
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
