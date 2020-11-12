import { polyfill, plugin_polyfill } from "./polyfills";
import { register_element } from "./register-element";
import { RufflePlayer } from "./ruffle-player";

/**
 * Represents this particular version of Ruffle.
 *
 * Multiple APIs can be instantiated from different sources; e.g. an "extension"
 * version, versus a "local" version. This expresses to the Public API
 * negotiator (see `PublicAPI`) what this particular version of Ruffle is and
 * how to control it.
 */
export class SourceAPI {
    private name: string;
    /**
     * Construct a Source API.
     *
     * @param {string} source_name The name of this particular source.
     */
    constructor(source_name: string) {
        this.name = source_name;
    }

    get version() {
        return "0.1.0";
    }

    /**
     * Start up the polyfills.
     *
     * Do not run polyfills for more than one Ruffle source at a time.
     *
     */
    polyfill() {
        polyfill();
    }
    /**
     * Polyfill the plugin detection.
     *
     * This needs to run before any plugin detection script does.
     *
     */
    plugin_polyfill() {
        plugin_polyfill();
    }

    /**
     * Create a Ruffle player element using this particular version of Ruffle.
     *
     * @returns {RufflePlayer} The player element. This is a DOM element that
     * may be inserted into the current page as you wish.
     */
    create_player() {
        const player_element_name = register_element(
            "ruffle-player",
            RufflePlayer
        );
        const player = document.createElement(player_element_name);

        return player;
    }
}
