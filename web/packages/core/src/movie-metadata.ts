/**
 * Metadata about a loaded SWF file.
 */
export interface MovieMetadata {
    /**
     * The width of the movie in pixels.
     */
    readonly width: number;

    /**
     * The height of the movie in pixels.
     */
    readonly height: number;

    /**
     * The frame rate of the movie in frames per second.
     */
    readonly frameRate: number;

    /**
     * The number of frames on the root timeline of the movie.
     */
    readonly numFrames: number;

    /**
     * The SWF version of the movie.
     */
    readonly swfVersion: number;
}
