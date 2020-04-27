/**
 * Pierce the extension sandbox by copying our code into window space.
 * 
 * The isolation extension content scripts get is neat, but it causes problems
 * based on what browser you use:
 * 
 * 1. On Chrome, you are explicitly banned from registering custom elements
 * 2. On Firefox, you can register custom elements but they can't expose any
 *    useful API surface, and can't even see their own methods.
 * 
 * This code exists to pierce the extension sandbox, while maintaining:
 * 
 * 1. The isolation of not interfering with the page's execution environment
 *    unintentionally.
 * 2. The ability to load extension resources such as .wasm files
 */
function insert_ruffle(mutationsList,observer) {
    let nodesAdded = mutationsList.some(mutation => mutation.addedNodes.length > 0);
    if (nodesAdded&&document.head) {
        let setup_scriptelem = document.createElement("script");
        let setup_src = "var runtime_path = \"" +
            ext_path + "\";\nvar obfuscated_event_prefix = \"" +
            obfuscated_event_prefix + "\";";
        let scriptelem = document.createElement("script");
        setup_scriptelem.appendChild(document.createTextNode(setup_src));
        document.head.appendChild(setup_scriptelem);
        scriptelem.src=ext_path + "dist/ruffle.js";
        document.head.appendChild(scriptelem);
        observer.disconnect();
    }
}

let page_optout = document.getElementsByTagName("html")[0].dataset.ruffleOptout !== undefined;
let obfuscated_event_prefix = "rufEvent" + Math.floor(Math.random() * 100000000000);
let ext_path = "";
if (chrome && chrome.extension && chrome.extension.getURL) {
    ext_path = chrome.extension.getURL("dist/ruffle.js").replace("dist/ruffle.js", "");
} else if (browser && browser.runtime && browser.runtime.getURL) {
    ext_path = browser.runtime.getURL("dist/ruffle.js").replace("dist/ruffle.js", "");
}
if (!(page_optout||window.RufflePlayer)) {
    const observer = new MutationObserver(insert_ruffle);
    observer.observe(document, {childList: true, subtree: true});
}
