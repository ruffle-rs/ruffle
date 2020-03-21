import RuffleObject from "./ruffle-object";
import RuffleEmbed from "./ruffle-embed";
import { install_plugin, FLASH_PLUGIN } from "./plugin-polyfill";

/**
 * Polyfill native elements with Ruffle equivalents.
 * 
 * This polyfill isn't fool-proof: If there's a chance site JavaScript has
 * access to a pre-polyfill element, then this will break horribly. We can
 * keep native objects out of the DOM, and thus out of JavaScript's grubby
 * little hands, but only if we load first.
 */
function wrap_tree(elem) {
    try {
        if (elem.nodeName.toLowerCase() === "object" && RuffleObject.is_interdictable(elem)) {
            let ruffle_obj = RuffleObject.from_native_object_element(elem);
            elem.parentElement.replaceChild(ruffle_obj, elem);
        } else if (elem.nodeName.toLowerCase() === "embed" && RuffleEmbed.is_interdictable(elem)) {
            let ruffle_obj = RuffleEmbed.from_native_object_element(elem);
            elem.parentElement.replaceChild(ruffle_obj, elem);
        } else {
            for (let node of Array.from(elem.getElementsByTagName("object"))) {
                if (RuffleObject.is_interdictable(node)) {
                    let ruffle_obj = RuffleObject.from_native_object_element(node);
                    node.parentElement.replaceChild(ruffle_obj, node);
                }
            }

            for (let node of Array.from(elem.getElementsByTagName("embed"))) {
                if (RuffleEmbed.is_interdictable(node)) {
                    let ruffle_obj = RuffleEmbed.from_native_embed_element(node);
                    node.parentElement.replaceChild(ruffle_obj, node);
                }
            }
        }
    } catch (err) {
        console.error("Serious error encountered when polyfilling native Flash elements: " + err);
    }
}

function polyfill_static_content() {
    wrap_tree(document.getElementsByTagName("html")[0]);
}

function dynamic_content_listener(event) {
    /* For use with polyfill_dynamic_content */
    var target=event.target;
    if (event.animationName == "ruffleObjectOrEmbedInserted") {
        /* Exclude embeds & objects inside of objects */
        if(target.parentElement.tagName.toLowerCase()!="object") {
            wrap_tree(target);
        }
    }
}

function polyfill_dynamic_content()
{
    /* This creates a stylesheet to apply a very short CSS animation to     *
     * object and embed tags, then detect the animation being applied       */
    var style=document.createElement("style");
    var sheet;
    var dynamic_content_listener;
    /* To make it work for webkit?  */
    style.appendChild(document.createTextNode(""));
    /* Add it to the page. */
    document.head.appendChild(style);
    sheet=style.sheet;
    /* The animation(opacity from 0.99 to 1) */
    sheet.insertRule("@keyframes ruffleObjectOrEmbedInserted { from { opacity: 0.99; } to { opacity: 1; } }",-1);
    /* Apply it to object & embed tags */
    sheet.insertRule("object, embed { animation-duration:0.001s; animation-name: ruffleObjectOrEmbedInserted; }",-1);
    document.addEventListener("animationstart", dynamic_content_listener, false); /* standard */
    document.addEventListener("MSAnimationStart", dynamic_content_listener, false); /* IE */
    document.addEventListener("webkitAnimationStart", dynamic_content_listener, false); /* Chrome + Safari */

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
