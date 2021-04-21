import { PublicAPI, SourceAPI, publicPath } from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "local",
    new SourceAPI("local")
);
__webpack_public_path__ = publicPath(window.RufflePlayer.config, "local");
const ruffle = window.RufflePlayer.newest();

let player;

// Default config used by the player.
const config = {
    letterbox: "on",
    logLevel: "warn",
};

window.addEventListener("DOMContentLoaded", () => {
    const url = new URL(window.location);
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

    player = ruffle.createPlayer();
    player.id = "player";
    document.getElementById("main").append(player);

    player.load({ url: swfUrl, ...config });
});
