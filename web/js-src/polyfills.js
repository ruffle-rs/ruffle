import RuffleObject from "./ruffle-object";
import RuffleEmbed from "./ruffle-embed";
import { install_plugin, FLASH_PLUGIN } from "./plugin-polyfill";

// Live collection for object and embed tags.
let objects = null;
let embeds = null;

/**
 * Polyfill native elements with Ruffle equivalents.
 * 
 * This polyfill isn't fool-proof: If there's a chance site JavaScript has
 * access to a pre-polyfill element, then this will break horribly. We can
 * keep native objects out of the DOM, and thus out of JavaScript's grubby
 * little hands, but only if we load first.
 */
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
        console.error("Serious error encountered when polyfilling native Flash elements: " + err);
    }
}

function polyfill_static_content() {
    replace_flash_instances();
}


function polyfill_dynamic_content() {
    // Listen for changes to the DOM. If nodes are added, re-check for any Flash instances.
    const observer = new MutationObserver(function (mutationsList, observer) {
        // If any nodes were added, re-run the polyfill to replace any new instances.
        let nodesAdded = mutationsList.some(mutation => mutation.addedNodes.length > 0);
        if (nodesAdded) {
            replace_flash_instances();
        }
    });

    observer.observe(document, { childList: true, subtree: true});
}

function falsify_plugin_detection() {
    install_plugin(FLASH_PLUGIN);
}

var running_polyfills = [];
var polyfills = {
    "static-content": polyfill_static_content,
    "dynamic-content": polyfill_dynamic_content,
    "plugin-detect": falsify_plugin_detection
};

export function polyfill(polyfill_list) {
    for (var i = 0; i < polyfill_list.length; i += 1) {
        if (running_polyfills.indexOf(polyfill_list[i]) !== -1) {
            continue;
        }

        if (!polyfills.hasOwnProperty(polyfill_list[i])) {
            throw new Error("Requested nonexistent polyfill: " + polyfill_list[i]);
        }

        running_polyfills.push(polyfill_list[i]);

        var this_polyfill = polyfills[polyfill_list[i]];

        if (this_polyfill.dependencies !== undefined) {
            polyfill(this_polyfill.dependencies);
        }

        this_polyfill();
    }
}