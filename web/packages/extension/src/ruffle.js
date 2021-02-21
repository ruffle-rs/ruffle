import { PublicAPI, SourceAPI, publicPath } from "ruffle-core";

function handleMessage(message) {
    // Ping back.
    return message;
}

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "extension",
    new SourceAPI("extension")
);
__webpack_public_path__ = publicPath(window.RufflePlayer.config, "extension");

window.addEventListener("message", async (event) => {
    // We only accept messages from ourselves.
    if (event.source !== window) {
        return;
    }

    const { type, index, data } = event.data;
    if (type === "FROM_RUFFLE") {
        const response = await handleMessage(data);
        window.postMessage({ type: "TO_RUFFLE", index, data: response }, "*");
    }
});
