import { pluginPolyfill, polyfill } from "./polyfills";
import { registerElement } from "./internal/register-element";
import { RufflePlayerElement } from "./internal/player/ruffle-player-element";
import { buildInfo } from "./build-info";
import { InstallationOptions } from "./install";
import { Player } from "./public/player";
import { OriginAPI } from "./origin-api";

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
     * The OriginAPI of this Ruffle version.
     * It is used to access behaviour that's specific to the Ruffle origin (selfhosted, demo or extension).
     * It's given to the (internal) Ruffle player and provides minor helper methods (unlike this SourceAPI,
     * which manages the whole Ruffle polyfill and player).
     */
    originAPI: OriginAPI;

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

let installationOptions: InstallationOptions = {} as InstallationOptions;

/**
 * Returns the actual source API implementation that describes this installation.
 * The internal implementation details aren't part of the public API and may contain extra details.
 *
 * @param originAPI The OriginAPI of this Ruffle installation.
 * @param options The InstallationOptions of this Ruffle installation.
 * @returns The actual source API implementation that describes this installation.
 */
export function getSourceApiImplementation(
    originAPI: OriginAPI,
    options: InstallationOptions,
) {
    installationOptions = options;

    return {
        /**
         * The version of this particular API, as a string in a semver compatible format.
         */
        version:
            buildInfo.versionNumber +
            "+" +
            buildInfo.buildDate.substring(0, 10),

        originAPI: originAPI,

        /**
         * Start up the polyfills.
         *
         * Do not run polyfills for more than one Ruffle source at a time.
         */
        polyfill(): void {
            polyfill(originAPI);
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
            const player = document.createElement(name) as RufflePlayerElement;
            player.initialize(originAPI);
            return player;
        },

        /**
         * Options specified by the user of this library.
         */
        options: installationOptions,
    };
}

/**
 * Executes the onFirstLoad method when called for the first time.
 */
export function onFirstLoad() {
    installationOptions.onFirstLoad?.();
    installationOptions.onFirstLoad = () => {};
}
