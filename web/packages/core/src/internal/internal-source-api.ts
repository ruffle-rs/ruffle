import { buildInfo } from "../build-info";
import { pluginPolyfill, polyfill } from "../polyfills";
import { PlayerElement } from "../public/player";
import { registerElement } from "./register-element";
import { RufflePlayerElement } from "./player/ruffle-player-element";
import { InstallationOptions } from "../public/setup";

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
    createPlayer(): PlayerElement {
        const name = registerElement("ruffle-player", RufflePlayerElement);
        return document.createElement(name) as RufflePlayerElement;
    },

    /**
     * Options specified by the user of this library.
     */
    options: {} as InstallationOptions,
};
