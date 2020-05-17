const { PublicAPI, SourceAPI, public_path } = require("ruffle-core");

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "local",
    new SourceAPI("local")
);
__webpack_public_path__ = public_path(window.RufflePlayer.config, "local");
