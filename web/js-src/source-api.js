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
    
    init(interdictions) {
        interdict(interdictions);
    }
}