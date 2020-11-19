import { pluginPolyfill, polyfill } from "./polyfills";
import { registerElement } from "./register-element";
import { RufflePlayer } from "./ruffle-player";

/**
 * Represents this particular version of Ruffle.
 *
 * Multiple APIs can be instantiated from different sources; e.g. an "extension"
 * version, versus a "local" version. This expresses to the Public API
 * negotiator (see [[PublicAPI]]) what this particular version of Ruffle is and
 * how to control it.
 */
export class SourceAPI {
    private name: string;

    /**
     * Construct a Source API.
     *
     * @param name The name of this particular source.
     */
    constructor(name: string) {
        this.name = name;
    }

    /**
     * The version of this particular API.
     *
     * This is returned as a string in a semver compatible format.
     *
     * @returns The version of this Ruffle source
     */
    get version(): string {
        return "%VERSION_NUMBER%";
    }

    /**
     * Start up the polyfills.
     *
     * Do not run polyfills for more than one Ruffle source at a time.
     */
    polyfill(): void {
        polyfill();
    }

    /**
     * Polyfill the plugin detection.
     *
     * This needs to run before any plugin detection script does.
     */
    pluginPolyfill(): void {
        pluginPolyfill();
    }

    /**
     * Create a Ruffle player element using this particular version of Ruffle.
     *
     * @returns The player element. This is a DOM element that may be inserted
     * into the current page as you wish.
     */
    createPlayer(): RufflePlayer {
        const name = registerElement("ruffle-player", RufflePlayer);
        return <RufflePlayer>document.createElement(name);
    }
}
