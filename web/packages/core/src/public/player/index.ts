/**
 * The Player module contains the actual {@link PlayerElement} and the various interfaces that exist to interact with the player.
 *
 * @module
 */

export * from "./flash";
export * from "./player-element";
export * from "./movie-metadata";
export * from "./legacy";
export * from "./v1";

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
