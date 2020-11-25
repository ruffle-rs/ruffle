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
