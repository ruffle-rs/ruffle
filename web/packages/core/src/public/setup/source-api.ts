import { PlayerElement } from "../player";

/**
 * Represents this particular version of Ruffle.
 *
 * Multiple APIs can be instantiated from different sources; e.g. an "extension"
 * version, versus a "local" version. This expresses to the Public API
 * negotiator (see {@link PublicAPI}) what this particular version of Ruffle is and
 * how to control it.
 */
export interface SourceAPI {
    /**
     * The version of this particular API, as a string in a semver compatible format.
     */
    version: string;

    /**
     * Start up the polyfills.
     *
     * Do not run polyfills for more than one Ruffle source at a time.
     */
    polyfill(): void;

    /**
     * Polyfill the plugin detection.
     *
     * This needs to run before any plugin detection script does.
     */
    pluginPolyfill(): void;

    /**
     * Create a Ruffle player element using this particular version of Ruffle.
     *
     * @returns The player element. This is a DOM element that may be inserted
     * into the current page as you wish.
     */
    createPlayer(): PlayerElement;
}
