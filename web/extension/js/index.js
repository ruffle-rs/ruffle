import { PublicAPI } from "../../js-src/public-api";
import { SourceAPI } from "../../js-src/source-api";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "extension",
    new SourceAPI()
);
