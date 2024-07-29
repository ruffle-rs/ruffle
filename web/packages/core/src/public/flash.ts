/**
 * These are properties and methods that Flash added to its <embed>/<object> tags.
 * These don't seem to be documented in full anywhere, and Ruffle adds them as we encounter some.
 * You are discouraged from using these, and they exist only to support legacy websites from decades ago.
 *
 * Extra methods or properties may appear at any time, due to `ExternalInterface.addCallback()`.
 * It may even overwrite existing methods or properties.
 */
export interface FlashAPI {
    /**
     * Returns the movies loaded process, in a percent from 0 to 100.
     * Ruffle may just return 0 or 100.
     *
     * @returns a value from 0 to 100, inclusive.
     */
    PercentLoaded(): number;
}
