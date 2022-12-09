import { pluginPolyfill, polyfill } from "./polyfills";
import { registerElement } from "./register-element";
import { RufflePlayer } from "./ruffle-player";
import { buildInfo } from "./build-info";

/**
 * Represents this particular version of Ruffle.
 *
 * Multiple APIs can be instantiated from different sources; e.g. an "extension"
 * version, versus a "local" version. This expresses to the Public API
 * negotiator (see [[PublicAPI]]) what this particular version of Ruffle is and
 * how to control it.
 */
export const SourceAPI = {
    /**
     * The version of this particular API, as a string in a semver compatible format.
     */
    version: buildInfo.versionNumber,

    /**
     * Start up the polyfills.
     *
     * Do not run polyfills for more than one Ruffle source at a time.
     *
     * @param isExt Whether or not Ruffle is running as a browser's extension.
     */
    polyfill(isExt: boolean): void {
        polyfill(isExt);
    },

    /**
     * Polyfill the plugin detection.
     *
     * This needs to run before any plugin detection script does.
     */
    pluginPolyfill(): void {
        pluginPolyfill();
    },

    /**
     * Create a Ruffle player element using this particular version of Ruffle.
     *
     * @returns The player element. This is a DOM element that may be inserted
     * into the current page as you wish.
     */
    createPlayer(): RufflePlayer {
        const name = registerElement("ruffle-player", RufflePlayer);
        return <RufflePlayer>document.createElement(name);
    },
};
