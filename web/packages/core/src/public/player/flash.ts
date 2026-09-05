/**
 * These are properties and methods that Flash added to its `<embed>/<object>` tags.
 * These don't seem to be documented in full anywhere, and Ruffle adds them as we encounter some.
 * You are discouraged from using these, and they exist only to support legacy websites from decades ago.
 *
 * https://web.archive.org/web/20100710000820/http://www.adobe.com/support/flash/publishexport/scriptingwithflash/scriptingwithflash_03.html
 *
 * Extra methods or properties may appear at any time, due to `ExternalInterface.addCallback()`.
 * It may even overwrite existing methods or properties.
 */
export interface FlashAPI {
    /**
     * Returns the current frame index of the movie.
     *
     * @remarks the frame number starts from a 0 index, not 1 like in FP
     * @returns the current frame of the movie.
     */
    CurrentFrame(): number;

    /**
     * Jumps the movie to a specific frame and stops.
     *
     * @remarks the frame number starts from a 0 index, not 1 like in FP
     */
    GotoFrame(frame: number): void;

    /**
     * Returns whether the movie's root clip is playing or not.
     * Not to be confused with whether Ruffle is playing.
     *
     * @returns true if the movie is playing, otherwise false.
     */
    IsPlaying(): boolean;

    /**
     * Returns the movies loaded process, in a percent from 0 to 100.
     * Ruffle may just return 0 or 100.
     *
     * @returns a value from 0 to 100, inclusive.
     */
    PercentLoaded(): number;

    /**
     * Starts playing the root movie clip of the movie.
     */
    Play(): void;

    /**
     * Stops the root movie clip of the movie.
     */
    StopPlay(): void;

    /**
     * Returns the movies loaded process, in a percent from 0 to 100.
     * Ruffle may just return 0 or 100.
     *
     * @returns the total number of frames in the movie.
     */
    TotalFrames(): number;
}
