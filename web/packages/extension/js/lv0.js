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
const {
    getSyncStorage,
    setMessageListener,
    getExtensionUrl,
} = require("./util.js");

getSyncStorage(["ruffleEnable", "ignoreOptout"], function (data) {
    let pageOptout = document.documentElement.hasAttribute(
        "data-ruffle-optout"
    );
    try {
        if (
            !pageOptout &&
            window.top &&
            window.top.document &&
            window.top.document.documentElement
        ) {
            /* In case the opting out page uses iframes */
            pageOptout = window.top.document.documentElement.hasAttribute(
                "data-ruffle-optout"
            );
        }
    } catch (e) {
        console.log("Unable to check top-level optout: " + e.message);
    }

    let shouldLoadUntrustedWorld = !(pageOptout || window.RufflePlayer);
    let obfuscatedEventPrefix =
        "rufEvent" + Math.floor(Math.random() * 100000000000);
    let nextResponsePromise = null;
    let nextResponsePromiseResolve = null;

    if (data) {
        shouldLoadUntrustedWorld =
            data.ruffleEnable &&
            !((pageOptout && !data.ignoreOptout) || window.RufflePlayer);
    } else {
        console.log("Couldn't read settings.");
    }

    document.addEventListener(
        obfuscatedEventPrefix + "_response",
        function (e) {
            if (nextResponsePromiseResolve !== null) {
                nextResponsePromiseResolve(e);

                nextResponsePromise = null;
                nextResponsePromiseResolve = null;
            }
        }
    );

    /**
     * Returns a promise which resolves the next time we receive our custom
     * event response.
     */
    function nextResponse() {
        if (nextResponsePromise == null) {
            nextResponsePromise = new Promise(function (resolve) {
                nextResponsePromiseResolve = resolve;
            });
        }

        return nextResponsePromise;
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
    async function marshalMessageIntoUntrustedWorld(request) {
        let reqEvent = new CustomEvent(obfuscatedEventPrefix + "_request", {
            detail: JSON.stringify(request),
        });
        let respEventHandler = nextResponse();

        document.dispatchEvent(reqEvent);

        let respEvent = await respEventHandler;
        return JSON.parse(respEvent.detail);
    }

    setMessageListener(function (request, sender, responseCallback) {
        if (shouldLoadUntrustedWorld) {
            let responsePromise = marshalMessageIntoUntrustedWorld(request);
            responsePromise
                .then(function (response) {
                    responseCallback({
                        loaded: true,
                        tabSettings: data,
                        optout: pageOptout,
                        untrustedResponse: response,
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
            responseCallback({
                loaded: false,
                tabSettings: data,
                optout: pageOptout,
            });

            return false;
        }
    });

    const extPath = getExtensionUrl();

    if (shouldLoadUntrustedWorld) {
        // We must run the plugin polyfill before any flash detection scripts.
        // Unfortunately, this might still be too late for some websites when using Chrome (issue #969).
        let polyfillScript = document.createElement("script");
        polyfillScript.innerHTML =
            '(function(){class RuffleMimeType{constructor(a,b,c){this.type=a,this.description=b,this.suffixes=c}}class RuffleMimeTypeArray{constructor(a){this.__mimetypes=[],this.__named_mimetypes={};for(let b of a)this.install(b)}install(a){let b=this.__mimetypes.length;this.__mimetypes.push(a),this.__named_mimetypes[a.type]=a,this[a.type]=a,this[b]=a}item(a){return this.__mimetypes[a]}namedItem(a){return this.__named_mimetypes[a]}get length(){return this.__mimetypes.length}}class RufflePlugin extends RuffleMimeTypeArray{constructor(a,b,c,d){super(d),this.name=a,this.description=b,this.filename=c}install(a){a.enabledPlugin||(a.enabledPlugin=this),super.install(a)}}class RufflePluginArray{constructor(a){this.__plugins=[],this.__named_plugins={};for(let b of a)this.install(b)}install(a){let b=this.__plugins.length;this.__plugins.push(a),this.__named_plugins[a.name]=a,this[a.name]=a,this[b]=a}item(a){return this.__plugins[a]}namedItem(a){return this.__named_plugins[a]}get length(){return this.__plugins.length}}const FLASH_PLUGIN=new RufflePlugin("Shockwave Flash","Shockwave Flash 32.0 r0","ruffle.js",[new RuffleMimeType("application/futuresplash","Shockwave Flash","spl"),new RuffleMimeType("application/x-shockwave-flash","Shockwave Flash","swf"),new RuffleMimeType("application/x-shockwave-flash2-preview","Shockwave Flash","swf"),new RuffleMimeType("application/vnd.adobe.flash-movie","Shockwave Flash","swf")]);function install_plugin(a){navigator.plugins.install||Object.defineProperty(navigator,"plugins",{value:new RufflePluginArray(navigator.plugins),writable:!1}),navigator.plugins.install(a),0<a.length&&!navigator.mimeTypes.install&&Object.defineProperty(navigator,"mimeTypes",{value:new RuffleMimeTypeArray(navigator.mimeTypes),writable:!1});for(var b=0;b<a.length;b+=1)navigator.mimeTypes.install(a[b])}install_plugin(FLASH_PLUGIN);})();';
        (document.head || document.documentElement).appendChild(polyfillScript);

        // Load Ruffle script asynchronously. By doing so, we can inject extra variables and isolate them from the global scope.
        (async function () {
            let ruffleSrcResp = await fetch(extPath + "dist/ruffle.js");
            if (ruffleSrcResp.ok) {
                let ruffleSource =
                    '(function () { var ruffleRuntimePath = "' +
                    extPath +
                    '";\nvar obfuscatedEventPrefix = "' +
                    obfuscatedEventPrefix +
                    '";\n' +
                    (await ruffleSrcResp.text()) +
                    "}())";
                let ruffleScript = document.createElement("script");
                ruffleScript.appendChild(document.createTextNode(ruffleSource));
                (document.head || document.documentElement).appendChild(
                    ruffleScript
                );
            } else {
                console.error("Critical error loading Ruffle into page");
            }
        })();
    }
});
