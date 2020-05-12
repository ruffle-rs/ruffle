import { PublicAPI } from "../../js-src/public-api";
import { SourceAPI } from "../../js-src/source-api";
import { public_path } from "../../js-src/public-path";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "local",
    new SourceAPI("local")
);
__webpack_public_path__ = public_path(window.RufflePlayer.config, "local");
