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
 *
 * We also provide a content script message listener that sends messages into
 * the untrusted world via custom events. We use a dynamically generated event
 * prefix that is only shared within the injected closure. This isn't entirely
 * foolproof, but is designed to
 */
chrome.storage.sync.get(["ruffle_enable", "ignore_optout"], function (data) {
    let page_optout = document.documentElement.hasAttribute(
        "data-ruffle-optout"
    );
    try {
        if (
            !page_optout &&
            window.top &&
            window.top.document &&
            window.top.document.documentElement
        ) {
            /* In case the opting out page uses iframes */
            page_optout = window.top.document.documentElement.hasAttribute(
                "data-ruffle-optout"
            );
        }
    } catch (e) {
        console.log("Unable to check top-level optout: " + e.message);
    }

    let should_load_untrusted_world = !(page_optout || window.RufflePlayer);
    let obfuscated_event_prefix =
        "rufEvent" + Math.floor(Math.random() * 100000000000);
    let next_response_promise = null;
    let next_response_promise_resolve = null;

    if (data) {
        should_load_untrusted_world =
            data.ruffle_enable === "on" &&
            !(
                (page_optout && data.ignore_optout !== "on") ||
                window.RufflePlayer
            );
    } else {
        console.log("Couldn't read settings.");
    }

    document.addEventListener(obfuscated_event_prefix + "_response", function (
        e
    ) {
        if (next_response_promise_resolve !== null) {
            next_response_promise_resolve(e);

            next_response_promise = null;
            next_response_promise_resolve = null;
        }
    });

    /**
     * Returns a promise which resolves the next time we receive our custom
     * event response.
     */
    function next_response() {
        if (next_response_promise == null) {
            next_response_promise = new Promise(function (resolve) {
                next_response_promise_resolve = resolve;
            });
        }

        return next_response_promise;
    }

    /**
     * Given a trusted request object, send it to the untrusted world and return
     * the response.
     *
     * Due to extension sandboxing (V8 worlds / Gecko Xrays) all data passed
     * into and out of the untrusted world must be plain data.
     *
     * @param request The request data from the background/popup page
     * @returns Response data to reply to the sender with.
     */
    async function marshal_message_into_untrusted_world(request) {
        let req_event = new CustomEvent(obfuscated_event_prefix + "_request", {
            detail: JSON.stringify(request),
        });
        let resp_event_handler = next_response();

        document.dispatchEvent(req_event);

        let resp_event = await resp_event_handler;
        console.log(resp_event.detail);
        return JSON.parse(resp_event.detail);
    }

    chrome.runtime.onMessage.addListener(function (
        request,
        sender,
        response_callback
    ) {
        if (should_load_untrusted_world) {
            let response_promise = marshal_message_into_untrusted_world(
                request
            );
            response_promise
                .then(function (response) {
                    response_callback({
                        loaded: true,
                        tab_settings: data,
                        optout: page_optout,
                        untrusted_response: response,
                    });
                })
                .catch(function (e) {
                    console.error(
                        "Error when marshalling tab message into untrusted world: " +
                            e
                    );
                    throw e;
                });

            return true;
        } else {
            response_callback({
                loaded: false,
                tab_settings: data,
                optout: page_optout,
            });

            return false;
        }
    });

    let ext_path = "";
    if (chrome && chrome.extension && chrome.extension.getURL) {
        ext_path = chrome.extension
            .getURL("dist/ruffle.js")
            .replace("dist/ruffle.js", "");
    } else if (browser && browser.runtime && browser.runtime.getURL) {
        ext_path = browser.runtime
            .getURL("dist/ruffle.js")
            .replace("dist/ruffle.js", "");
    }

    if (should_load_untrusted_world) {
        let setup_scriptelem = document.createElement("script");
        let setup_src =
            'var runtime_path = "' +
            ext_path +
            '";\nvar obfuscated_event_prefix = "' +
            obfuscated_event_prefix +
            '";' +
            '(function(){class RuffleMimeType{constructor(a,b,c){this.type=a,this.description=b,this.suffixes=c}}class RuffleMimeTypeArray{constructor(a){this.__mimetypes=[],this.__named_mimetypes={};for(let b of a)this.install(b)}install(a){let b=this.__mimetypes.length;this.__mimetypes.push(a),this.__named_mimetypes[a.type]=a,this[a.type]=a,this[b]=a}item(a){return this.__mimetypes[a]}namedItem(a){return this.__named_mimetypes[a]}get length(){return this.__mimetypes.length}}class RufflePlugin extends RuffleMimeTypeArray{constructor(a,b,c,d){super(d),this.name=a,this.description=b,this.filename=c}install(a){a.enabledPlugin||(a.enabledPlugin=this),super.install(a)}}class RufflePluginArray{constructor(a){this.__plugins=[],this.__named_plugins={};for(let b of a)this.install(b)}install(a){let b=this.__plugins.length;this.__plugins.push(a),this.__named_plugins[a.name]=a,this[a.name]=a,this[b]=a}item(a){return this.__plugins[a]}namedItem(a){return this.__named_plugins[a]}get length(){return this.__plugins.length}}const FLASH_PLUGIN=new RufflePlugin("Shockwave Flash","Shockwave Flash 32.0 r0","ruffle.js",[new RuffleMimeType("application/futuresplash","Shockwave Flash","spl"),new RuffleMimeType("application/x-shockwave-flash","Shockwave Flash","swf"),new RuffleMimeType("application/x-shockwave-flash2-preview","Shockwave Flash","swf"),new RuffleMimeType("application/vnd.adobe.flash-movie","Shockwave Flash","swf")]);function install_plugin(a){navigator.plugins.install||Object.defineProperty(navigator,"plugins",{value:new RufflePluginArray(navigator.plugins),writable:!1}),navigator.plugins.install(a),0<a.length&&!navigator.mimeTypes.install&&Object.defineProperty(navigator,"mimeTypes",{value:new RuffleMimeTypeArray(navigator.mimeTypes),writable:!1});for(var b=0;b<a.length;b+=1)navigator.mimeTypes.install(a[b])}install_plugin(FLASH_PLUGIN);})();';
        let scriptelem = document.createElement("script");
        setup_scriptelem.innerHTML = setup_src;
        /* The setup_scriptelem sets runtime_path & obfuscated_event_prefix *
         * and runs the plugin polyfill, which must run before any flash    *
         * detection scripts.                                               */
        (document.head || document.documentElement).appendChild(
            setup_scriptelem
        );
        scriptelem.src = ext_path + "dist/ruffle.js";
        (document.head || document.documentElement).appendChild(scriptelem);
    }
});
