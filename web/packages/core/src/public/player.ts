import { LegacyRuffleAPI } from "./legacy";
import { FlashAPI } from "./flash";

/**
 * A Ruffle player's HTML element.
 *
 * This is either created through `window.RufflePlayer.createPlayer()`, or polyfilled from a `<embed>`/`<object>` tag.
 *
 * In addition to usual HTML attributes, this player contains methods and properties that belong to both
 * the **Flash JS API** and **legacy Ruffle API**s.
 */
export interface Player extends HTMLElement, LegacyRuffleAPI, FlashAPI {}
