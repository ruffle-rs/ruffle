import { PublicAPI } from "ruffle-core/public-api";
import { SourceAPI } from "ruffle-core/source-api";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "extension",
    new SourceAPI()
);
