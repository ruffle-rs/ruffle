import { PublicAPI, SourceAPI, publicPath } from "ruffle-core";

let ruffle;
let player;

// Default config used by the player.
const config = {
    letterbox: "on",
    logLevel: "warn",
};

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "local",
    new SourceAPI("local")
);
__webpack_public_path__ = publicPath(window.RufflePlayer.config, "local");

window.addEventListener("DOMContentLoaded", () => {
    const url = new URL(window.location);
    const swfUrl = url.searchParams.get("url");
    if (!swfUrl) {
        return;
    }

    ruffle = window.RufflePlayer.newest();
    player = ruffle.createPlayer();
    player.id = "player";
    document.getElementById("main").append(player);

    player.load({ url: swfUrl, ...config });
});
