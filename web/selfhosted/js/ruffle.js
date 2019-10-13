import { PublicAPI } from "../../js-src/public-api";
import { SourceAPI } from "../../js-src/source-api";

window.RufflePlayer = PublicAPI.negotiate(window.RufflePlayer, "local", new SourceAPI());