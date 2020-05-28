import { PublicAPI, SourceAPI } from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "extension",
    new SourceAPI()
);

if (obfuscated_event_prefix) {
    document.addEventListener(obfuscated_event_prefix + "_request", function (
        e
    ) {
        let body = JSON.parse(e.detail);
        let response = {};

        if (body.action === "get_page_options") {
            //response.page_options = page_options;
        }

        let event = new CustomEvent(obfuscated_event_prefix + "_response", {
            detail: JSON.stringify(response),
        });
        document.dispatchEvent(event);
    });
}
