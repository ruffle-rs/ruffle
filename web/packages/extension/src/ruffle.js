import { PublicAPI, SourceAPI, publicPath } from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "extension",
    new SourceAPI("extension")
);
__webpack_public_path__ = publicPath(window.RufflePlayer.config, "extension");
