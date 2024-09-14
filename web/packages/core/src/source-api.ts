import { pluginPolyfill, polyfill } from "./polyfills";
import { registerElement } from "./internal/register-element";
import { RufflePlayerElement } from "./internal/player/ruffle-player-element";
import { buildInfo } from "./build-info";
import { InstallationOptions } from "./install";
import { Player } from "./public/player";

/**
 * Represents this particular version of Ruffle.
 *
 * Multiple APIs can be instantiated from different sources; e.g. an "extension"
 * version, versus a "local" version. This expresses to the Public API
 * negotiator (see [[PublicAPI]]) what this particular version of Ruffle is and
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
    createPlayer(): Player;
}

/**
 * The actual source API that describes this installation.
 * This isn't part of the public API and may contain extra details.
 */
export const internalSourceApi = {
    /**
     * The version of this particular API, as a string in a semver compatible format.
     */
    version:
        buildInfo.versionNumber + "+" + buildInfo.buildDate.substring(0, 10),

    /**
     * Start up the polyfills.
     *
     * Do not run polyfills for more than one Ruffle source at a time.
     */
    polyfill(): void {
        polyfill();
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
    createPlayer(): Player {
        const name = registerElement("ruffle-player", RufflePlayerElement);
        return document.createElement(name) as RufflePlayerElement;
    },

    /**
     * Options specified by the user of this library.
     */
    options: {} as InstallationOptions,
};
