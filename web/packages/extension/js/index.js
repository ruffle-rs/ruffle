import { PublicAPI, SourceAPI } from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "extension",
    new SourceAPI("extension")
);
function make_event_handler() {
    if (window.obfuscated_event_prefix) {
        document.addEventListener(
            obfuscated_event_prefix + "_request",
            function (e) {
                let body = JSON.parse(e.detail);
                let response = {};

                if (body.action === "get_page_options") {
                    //response.page_options = page_options;
                }

                let event = new CustomEvent(
                    obfuscated_event_prefix + "_response",
                    {
                        detail: JSON.stringify(response),
                    }
                );
                document.dispatchEvent(event);
            }
        );
    } else {
        window.setTimeout(make_event_handler, 500);
    }
}
make_event_handler();
