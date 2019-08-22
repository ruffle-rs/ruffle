import RuffleObject from "./ruffle-object";
import RuffleEmbed from "./ruffle-embed";

/**
 * Interdict native elements with Ruffle equivalents.
 * 
 * This interdiction isn't fool-proof: If there's a chance site JavaScript has
 * access to a pre-interdiction element, then this will break horribly. We can
 * keep native objects out of the DOM, and thus out of JavaScript's grubby
 * little hands, but only if we load first.
 * 
 * The requirement to wait for WASM is a huge problem in practice.
 */
function wrap_tree(elem) {
    for (let node of elem.getElementsByTagName("object")) {
        if (RuffleObject.is_interdictable(node)) {
            let ruffle_obj = RuffleObject.from_native_object_element(node);
            node.parentElement.replaceChild(ruffle_obj, node);
        }
    }

    for (let node of elem.getElementsByTagName("embed")) {
        if (RuffleEmbed.is_interdictable(node)) {
            let ruffle_obj = RuffleEmbed.from_native_embed_element(node);
            node.parentElement.replaceChild(ruffle_obj, node);
        }
    }
}

console.log("Welcome to ruffle");

window.customElements.define("ruffle-object", RuffleObject);
window.customElements.define("ruffle-embed", RuffleEmbed);
wrap_tree(document.getElementsByTagName("html")[0]);

const observer = new MutationObserver(function (mutationsList, observer) {
    console.log(mutationsList);
    for (let mutation of mutationsList) {
        for (let node of mutation.addedNodes) {
            wrap_tree(node);
        }
    }
});

observer.observe(document, { childList: true, subtree: true});