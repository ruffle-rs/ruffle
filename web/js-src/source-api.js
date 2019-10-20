import { polyfill } from "./polyfills";
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
    /**
     * Construct a Source API.
     * 
     * @param {string} source_name The name of this particular source.
     */
    constructor(source_name) {
        this.name = name;
    }

    get version() {
        return "0.1.0";
    }
    
    /**
     * Start up a particular set of polyfills.
     * 
     * Polyfills, once enabled, may not be disabled. However, this function may
     * be called again with a different list to enable further polyfills.
     * 
     * Do not run polyfills for more than one Ruffle source at a time.
     * 
     * @param {array} polyfills A list of polyfills. See the `polyfills` module
     * for a list of allowable strings.
     */
    polyfill(polyfills) {
        polyfill(polyfills);
    }
    
    /**
     * Create a Ruffle player element using this particular version of Ruffle.
     * 
     * @returns {RufflePlayer} The player element. This is a DOM element that
     * may be inserted into the current page as you wish.
     */
    create_player() {
        let player_element_name = register_element("ruffle-player", RufflePlayer);
        let player = document.createElement(player_element_name);

        return player;
    }
}