import { RuffleObject } from "./ruffle-object";
import { RuffleEmbed } from "./ruffle-embed";
import { installPlugin, FLASH_PLUGIN } from "./plugin-polyfill";
import { publicPath } from "./public-path";
import { Config } from "./config";

if (!window.RufflePlayer) {
    window.RufflePlayer = {};
}
let topLevelRuffleConfig: Config;
let ruffleScriptSrc = publicPath({}, "ruffle.js");
if (window.RufflePlayer.config) {
    topLevelRuffleConfig = window.RufflePlayer.config;
    ruffleScriptSrc = publicPath(window.RufflePlayer.config, "ruffle.js");
}
/* public_path returns the directory where the file is, *
 * so we need to append the filename. We don't need to  *
 * worry about the directory not having a slash because *
 * public_path appends a slash.                         */
ruffleScriptSrc += "ruffle.js";

/**
 * Polyfill native elements with Ruffle equivalents.
 *
 * This polyfill isn't fool-proof: If there's a chance site JavaScript has
 * access to a pre-polyfill element, then this will break horribly. We can
 * keep native objects out of the DOM, and thus out of JavaScript's grubby
 * little hands, but only if we load first.
 */
let objects: HTMLCollectionOf<HTMLElement>;
let embeds: HTMLCollectionOf<HTMLElement>;
function replaceFlashInstances(): void {
    try {
        // Create live collections to track embed tags.
        objects = objects || document.getElementsByTagName("object");
        embeds = embeds || document.getElementsByTagName("embed");

        // Replace <object> first, because <object> often wraps <embed>.
        for (const elem of Array.from(objects)) {
            if (RuffleObject.isInterdictable(elem)) {
                const ruffleObject = RuffleObject.fromNativeObjectElement(elem);
                elem.replaceWith(ruffleObject);
            }
        }
        for (const elem of Array.from(embeds)) {
            if (RuffleEmbed.isInterdictable(elem)) {
                const ruffleObject = RuffleEmbed.fromNativeEmbedElement(elem);
                elem.replaceWith(ruffleObject);
            }
        }
    } catch (err) {
        console.error(
            "Serious error encountered when polyfilling native Flash elements: " +
                err
        );
    }
}

function polyfillStaticContent(): void {
    replaceFlashInstances();
}

function polyfillDynamicContent(): void {
    // Listen for changes to the DOM. If nodes are added, re-check for any Flash instances.
    const observer = new MutationObserver(function (mutationsList) {
        // If any nodes were added, re-run the polyfill to replace any new instances.
        const nodesAdded = mutationsList.some(
            (mutation) => mutation.addedNodes.length > 0
        );
        if (nodesAdded) {
            replaceFlashInstances();
        }
    });

    observer.observe(document, { childList: true, subtree: true });
}

function loadRufflePlayerIntoFrame(event: Event): void {
    const currentTarget = event.currentTarget;
    if (currentTarget != null && "contentWindow" in currentTarget) {
        loadFrame(currentTarget["contentWindow"]);
    }
}
function loadFrame(currentFrame: Window): void {
    let frameDocument;
    try {
        frameDocument = currentFrame.document;
        if (!frameDocument) {
            console.log("Frame has no document.");
            return;
        }
    } catch (e) {
        console.log("Error Getting Frame: " + e.message);
        return;
    }
    if (!currentFrame.RufflePlayer) {
        /* Make sure we populate the frame's window.RufflePlayer.config */
        currentFrame.RufflePlayer = {};
        currentFrame.RufflePlayer.config = topLevelRuffleConfig;
        const script = frameDocument.createElement("script");
        script.src = ruffleScriptSrc; /* Load this script(ruffle.js) into the frame */
        frameDocument.body.appendChild(script);
    } else {
        console.log("(i)frame already has RufflePlayer");
    }
    polyfillFramesCommon(currentFrame);
}

function handleFrames(
    frameList: HTMLCollectionOf<HTMLIFrameElement | HTMLFrameElement>
): void {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let originalOnLoad: ((ev: Event) => any) | null;
    for (let i = 0; i < frameList.length; i++) {
        const currentFrame = frameList[i];
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
            try {
                if (
                    currentFrame.contentDocument &&
                    currentFrame.contentDocument.readyState &&
                    currentFrame.contentDocument.readyState == "complete" &&
                    currentFrame.contentWindow
                ) {
                    loadFrame(currentFrame.contentWindow);
                }
            } catch (e) {
                console.log(
                    "error loading ruffle player into frame: " + e.message
                );
            }
        }, 500);
        try {
            if ((originalOnLoad = currentFrame.onload)) {
                currentFrame.onload = function (event) {
                    if (originalOnLoad != null) {
                        try {
                            originalOnLoad(event);
                        } catch (e) {
                            console.log(
                                "Error calling original onload: " + e.message
                            );
                        }
                    }
                    loadRufflePlayerIntoFrame(event);
                };
            } else {
                currentFrame.onload = loadRufflePlayerIntoFrame;
            }
            const depth = currentFrame.contentWindow;
            if (depth != null) {
                polyfillFramesCommon(depth);
            }
        } catch (e) {
            console.log("error loading ruffle player into frame: " + e.message);
        }
    }
}

function polyfillFramesCommon(depth: Window): void {
    handleFrames(depth.document.getElementsByTagName("iframe"));
    handleFrames(depth.document.getElementsByTagName("frame"));
}

function polyfillStaticFrames(): void {
    polyfillFramesCommon(window);
}

function runFrameListener(mutationsList: MutationRecord[]): void {
    /* Basically the same as the listener for dynamic embeds. */
    const nodesAdded = mutationsList.some(
        (mutation) => mutation.addedNodes.length > 0
    );
    if (nodesAdded) {
        polyfillFramesCommon(window);
    }
}

function polyfillDynamicFrames(): void {
    const observer = new MutationObserver(runFrameListener);
    observer.observe(document, { childList: true, subtree: true });
}

/**
 * Polyfills the detection of flash plugins in the browser.
 */
export function pluginPolyfill(): void {
    installPlugin(FLASH_PLUGIN);
}

/**
 * Polyfills legacy flash content on the page.
 */
export function polyfill(): void {
    polyfillStaticContent();
    polyfillDynamicContent();
    polyfillStaticFrames();
    polyfillDynamicFrames();
}
