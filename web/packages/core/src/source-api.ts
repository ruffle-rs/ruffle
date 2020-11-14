import { plugin_polyfill, polyfill } from "./polyfills";
import { register_element } from "./register-element";
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
     * @param source_name The name of this particular source.
     */
    constructor(source_name: string) {
        this.name = source_name;
    }

    /**
     * The version of this particular API.
     *
     * This is returned as a string in a semver compatible format.
     */
    get version(): string {
        return "0.1.0";
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
        plugin_polyfill();
    }

    /**
     * Create a Ruffle player element using this particular version of Ruffle.
     *
     * @returns The player element. This is a DOM element that may be inserted
     * into the current page as you wish.
     */
    create_player(): RufflePlayer {
        const player_element_name = register_element(
            "ruffle-player",
            RufflePlayer
        );
        return <RufflePlayer>document.createElement(player_element_name);
    }
}
