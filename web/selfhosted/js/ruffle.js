import { PublicAPI } from "ruffle-core/public-api";
import { SourceAPI } from "ruffle-core/source-api";
import { public_path } from "ruffle-core/public-path";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "local",
    new SourceAPI("local")
);
__webpack_public_path__ = public_path(window.RufflePlayer.config, "local");
