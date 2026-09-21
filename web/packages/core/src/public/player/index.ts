/**
 * The Player module contains the actual {@link PlayerElement} and the various interfaces that exist to interact with the player.
 *
 * @module
 */

export * from "./flash.js";
export * from "./player-element.js";
export * from "./movie-metadata.js";
export * from "./legacy.js";
export * from "./v1.js";

/**
 * Describes the loading state of an SWF movie.
 */
export enum ReadyState {
    /**
     * No movie is loaded, or no information is yet available about the movie.
     */
    HaveNothing = 0,

    /**
     * The movie is still loading, but it has started playback, and metadata is available.
     */
    Loading = 1,

    /**
     * The movie has completely loaded.
     */
    Loaded = 2,
}
