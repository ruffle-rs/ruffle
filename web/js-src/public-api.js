import { interdict } from "./interdiction";

/**
 * Construct a public API for this particular iteration of Ruffle for Web.
 */
export function construct_public_api() {
    return {
        "version": "0.1.0",
        "init": function (interdictions) {
            window.RufflePlayer.invoked = true;
            interdict(interdictions);
        }
    };
}