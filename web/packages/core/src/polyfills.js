const RuffleObject = require("./ruffle-object");
const RuffleEmbed = require("./ruffle-embed");
const { install_plugin, FLASH_PLUGIN } = require("./plugin-polyfill");
const { public_path } = require("./public-path");

if (!window.RufflePlayer) {
    window.RufflePlayer = {};
}
let top_level_ruffle_config;
let ruffle_script_src = public_path({}, "ruffle.js");
if (window.RufflePlayer.config) {
    top_level_ruffle_config = window.RufflePlayer.config;
    ruffle_script_src = public_path(window.RufflePlayer.config, "ruffle.js");
}
/* public_path returns the directory where the file is, *
 * so we need to append the filename. We don't need to  *
 * worry about the directory not having a slash because *
 * public_path appends a slash.                         */
ruffle_script_src += "ruffle.js";

/**
 * Polyfill native elements with Ruffle equivalents.
 *
 * This polyfill isn't fool-proof: If there's a chance site JavaScript has
 * access to a pre-polyfill element, then this will break horribly. We can
 * keep native objects out of the DOM, and thus out of JavaScript's grubby
 * little hands, but only if we load first.
 */
let objects;
let embeds;
function replace_flash_instances() {
    try {
        // Create live collections to track embed tags.
        objects = objects || document.getElementsByTagName("object");
        embeds = embeds || document.getElementsByTagName("embed");

        // Replace <object> first, because <object> often wraps <embed>.
        for (let elem of Array.from(objects)) {
            if (RuffleObject.is_interdictable(elem)) {
                let ruffle_obj = RuffleObject.from_native_object_element(elem);
                elem.replaceWith(ruffle_obj);
            }
        }
        for (let elem of Array.from(embeds)) {
            if (RuffleEmbed.is_interdictable(elem)) {
                let ruffle_obj = RuffleEmbed.from_native_embed_element(elem);
                elem.replaceWith(ruffle_obj);
            }
        }
    } catch (err) {
        console.error(
            "Serious error encountered when polyfilling native Flash elements: " +
                err
        );
    }
}

function polyfill_static_content() {
    replace_flash_instances();
}

function polyfill_dynamic_content() {
    // Listen for changes to the DOM. If nodes are added, re-check for any Flash instances.
    const observer = new MutationObserver(function (mutationsList) {
        // If any nodes were added, re-run the polyfill to replace any new instances.
        let nodesAdded = mutationsList.some(
            (mutation) => mutation.addedNodes.length > 0
        );
        if (nodesAdded) {
            replace_flash_instances();
        }
    });

    observer.observe(document, { childList: true, subtree: true });
}

function load_ruffle_player_into_frame(event) {
    loadFrame(event.currentTarget.contentWindow);
}
function loadFrame(current_frame) {
    let frame_document;
    try {
        frame_document = current_frame.document;
        if (!frame_document) {
            console.log("Frame has no document.");
            return;
        }
    } catch (e) {
        console.log("Error Getting Frame: " + e.message);
        return;
    }
    if (!current_frame.RufflePlayer) {
        /* Make sure we populate the frame's window.RufflePlayer.config */
        current_frame.RufflePlayer = {};
        current_frame.RufflePlayer.config = top_level_ruffle_config;
        let script = frame_document.createElement("script");
        script.src = ruffle_script_src; /* Load this script(ruffle.js) into the frame */
        frame_document.body.appendChild(script);
    } else {
        console.log("(i)frame already has RufflePlayer");
    }
    polyfill_frames_common(current_frame);
}

function handleFrames(frameList) {
    let originalOnLoad;
    for (let i = 0; i < frameList.length; i++) {
        let current_frame = frameList[i];
        /* Apparently, using addEventListener attaches the event  *
         * to the dummy document, which is overwritten when the   *
         * iframe is loaded, so we do this. It can only works if  *
         * it's attached to the frame object itself, which is why *
         * we're using                                            *
         * depth.document.getElementsByTagName("iframe") instead  *
         * of depth.frames to get the iframes at the depth.       *
         * Also, this way we should be able to handle frame       *
         * frame navigation, which is good.                       */
        setTimeout(function () {
            if (
                current_frame.contentDocument.readyState &&
                current_frame.contentDocument.readyState == "complete"
            ) {
                loadFrame(current_frame.contentWindow);
            }
        }, 500);
        if ((originalOnLoad = current_frame.onload)) {
            current_frame.onload = function (event) {
                originalOnLoad(event);
                load_ruffle_player_into_frame(event);
            };
        } else {
            current_frame.onload = load_ruffle_player_into_frame;
        }
        polyfill_frames_common(current_frame.contentWindow);
    }
}

function polyfill_frames_common(depth) {
    handleFrames(depth.document.getElementsByTagName("iframe"));
    handleFrames(depth.document.getElementsByTagName("frame"));
}

function polyfill_static_frames() {
    polyfill_frames_common(window);
}

function ruffle_frame_listener(mutationsList) {
    /* Basically the same as the listener for dynamic embeds. */
    let nodesAdded = mutationsList.some(
        (mutation) => mutation.addedNodes.length > 0
    );
    if (nodesAdded) {
        polyfill_frames_common(window);
    }
}

function polyfill_dynamic_frames() {
    const observer = new MutationObserver(ruffle_frame_listener);
    observer.observe(document, { childList: true, subtree: true });
}

exports.plugin_polyfill = function plugin_polyfill() {
    install_plugin(FLASH_PLUGIN);
};

exports.polyfill = function polyfill() {
    polyfill_static_content();
    polyfill_dynamic_content();
    polyfill_static_frames();
    polyfill_dynamic_frames();
};
