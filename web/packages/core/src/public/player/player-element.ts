import { LegacyRuffleAPI } from "./legacy";
import { FlashAPI } from "./flash";
import { PlayerV1 } from "./v1";

/**
 * A map of API version number, to API interface.
 */
export type APIVersions = {
    1: PlayerV1;
};

/**
 * A Ruffle player's HTML element.
 *
 * This is either created through `window.RufflePlayer.latest().createPlayer()`, or polyfilled from a `<embed>`/`<object>` tag.
 *
 * In addition to usual HTML attributes, this player contains methods and properties that belong to both
 * the **Flash JS API** and **legacy Ruffle API**s. You are strongly discouraged from using them, and should instead
 * use `.ruffle(version)` to access a versioned API interface.
 */
export interface PlayerElement extends HTMLElement, LegacyRuffleAPI, FlashAPI {
    /**
     * Access a specific version of the Ruffle API.
     * If the given version is not supported, an error is thrown.
     *
     * @param version Version of the API to access. Defaults to 1.
     */
    ruffle<V extends keyof APIVersions = 1>(version?: V): APIVersions[V];
}
