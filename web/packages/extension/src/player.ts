import * as utils from "./utils";
import { PublicAPI, SourceAPI, Letterbox } from "ruffle-core";

const api = PublicAPI.negotiate(
    window.RufflePlayer!,
    "local",
    new SourceAPI("local")
);
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

    const config = {
        letterbox: Letterbox.On,
        ...(await utils.getOptions(["warnOnUnsupportedContent", "logLevel"])),
    };
    player.load({ url: swfUrl, base: swfUrl, ...config });
});
