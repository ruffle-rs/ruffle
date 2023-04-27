import * as utils from "./utils";
import { PublicAPI } from "ruffle-core";
import type { Letterbox } from "ruffle-core";

const api = PublicAPI.negotiate(window.RufflePlayer!, "local");
window.RufflePlayer = api;
const ruffle = api.newest()!;

window.addEventListener("DOMContentLoaded", async () => {
    const url = new URL(window.location.href);
    const swfUrl = url.searchParams.get("url");
    if (!swfUrl) {
        return;
    }

    try {
        const pathname = new URL(swfUrl).pathname;
        document.title = pathname.substring(pathname.lastIndexOf("/") + 1);
    } catch (_) {
        // Ignore URL parsing errors.
    }

    const player = ruffle.createPlayer();
    player.id = "player";
    player.setIsExtension(true);
    document.getElementById("main")!.append(player);

    const options = await utils.getOptions();

    player.load({
        ...options,
        // Override default value for 'letterbox' when playing in the extension player page.
        letterbox: "on" as Letterbox,
        url: swfUrl,
        base: swfUrl.substring(0, swfUrl.lastIndexOf("/") + 1),
    });
});
