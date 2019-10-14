import { interdict } from "./interdiction";

/**
 * Represents this particular version of Ruffle.
 * 
 * Multiple APIs can be instantiated from different sources; e.g. an "extension"
 * version, versus a "local" version. This expresses to the Public API
 * negotiator (see `PublicAPI`) what this particular version of Ruffle is and
 * how to control it.
 */
export class SourceAPI {
    get version() {
        return "0.1.0";
    }
    
    /**
     * Start up a particular set of interdictions.
     * 
     * Interdictions, once enabled, may not be disabled. However, this function
     * may be called again with a different list to enable further
     * interdictions.
     * 
     * @param {array} interdictions A list of interdictions. See the
     * `interdiction` module for a list of allowable strings.
     */
    interdict(interdictions) {
        interdict(interdictions);
    }
}