import { RuffleObject } from "./ruffle-object";
import { RuffleEmbed } from "./ruffle-embed";
import { installPlugin, FLASH_PLUGIN } from "./plugin-polyfill";
import { publicPath } from "./public-path";
import type { Config } from "./config";

let isExtension: boolean;
const globalConfig: Config = window.RufflePlayer?.config ?? {};
const jsScriptUrl = publicPath(globalConfig) + "ruffle.js";

/**
 * Polyfill native Flash elements with Ruffle equivalents.
 *
 * This polyfill isn't fool-proof: If there's a chance site JavaScript has
 * access to a pre-polyfill element, then this will break horribly. We can
 * keep native objects out of the DOM, and thus out of JavaScript's grubby
 * little hands, but only if we load first.
 */
let objects: HTMLCollectionOf<HTMLObjectElement>;
let embeds: HTMLCollectionOf<HTMLEmbedElement>;
/**
 *
 */
function polyfillFlashInstances(): void {
    try {
        // Create live collections to track embed tags.
        objects = objects ?? document.getElementsByTagName("object");
        embeds = embeds ?? document.getElementsByTagName("embed");

        // Replace <object> first, because <object> often wraps <embed>.
        for (const elem of Array.from(objects)) {
            if (RuffleObject.isInterdictable(elem)) {
                const ruffleObject = RuffleObject.fromNativeObjectElement(elem);
                ruffleObject.setIsExtension(isExtension);
                elem.replaceWith(ruffleObject);
            }
        }
        for (const elem of Array.from(embeds)) {
            if (RuffleEmbed.isInterdictable(elem)) {
                const ruffleEmbed = RuffleEmbed.fromNativeEmbedElement(elem);
                ruffleEmbed.setIsExtension(isExtension);
                elem.replaceWith(ruffleEmbed);
            }
        }
    } catch (err) {
        console.error(
            `Serious error encountered when polyfilling native Flash elements: ${err}`
        );
    }
}

/**
 * Inject Ruffle into <iframe> and <frame> elements.
 *
 * This polyfill isn't fool-proof either: On self-hosted builds, it may
 * not work due to browsers CORS policy or be loaded too late for some
 * libraries like SWFObject. These should be less of a problem on the
 * web extension. This polyfill should, however, do the trick in most
 * cases, but users should be aware of its natural limits.
 */
let iframes: HTMLCollectionOf<HTMLIFrameElement>;
let frames: HTMLCollectionOf<HTMLFrameElement>;
/**
 *
 */
function polyfillFrames(): void {
    // Create live collections to track embed tags.
    iframes = iframes ?? document.getElementsByTagName("iframe");
    frames = frames ?? document.getElementsByTagName("frame");

    [iframes, frames].forEach((elementsList) => {
        for (let i = 0; i < elementsList.length; i++) {
            const element = elementsList[i];
            if (element.dataset.rufflePolyfilled !== undefined) {
                // Don't re-polyfill elements with the "data-ruffle-polyfilled" attribute.
                continue;
            }
            element.dataset.rufflePolyfilled = "";

            const elementWindow = element.contentWindow;

            // Cross origin requests may reach an exception, so let's prepare for this eventuality.
            const errorMessage = `Couldn't load Ruffle into ${element.tagName}[${element.src}]: `;
            try {
                if (elementWindow!.document!.readyState === "complete") {
                    injectRuffle(elementWindow!, errorMessage);
                }
            } catch (err) {
                if (!isExtension) {
                    // The web extension should be able to load Ruffle into cross origin frames
                    // because it has "all_frames" set to true in its manifest.json: RufflePlayer
                    // config won't be injected but it's not worth showing an error.
                    console.warn(errorMessage + err);
                }
            }

            // Attach listener to the element to handle frame navigation.
            element.addEventListener(
                "load",
                () => {
                    injectRuffle(elementWindow!, errorMessage);
                },
                false
            );
        }
    });
}

/**
 * @param elementWindow The (i)frame's window object.
 * @param errorMessage The message to log when Ruffle cannot access the (i)frame's document.
 */
async function injectRuffle(
    elementWindow: Window,
    errorMessage: string
): Promise<void> {
    // The document is supposed to be completely loaded when this function is run.
    // As Chrome may be unable to access the document properties, we have to delay the execution a little bit.
    await new Promise<void>((resolve) => {
        window.setTimeout(() => {
            resolve();
        }, 100);
    });

    let elementDocument: Document;
    try {
        elementDocument = elementWindow.document;
        if (!elementDocument) {
            // Don't polyfill if the window has no document: the element may have been removed from the parent window.
            return;
        }
    } catch (err) {
        if (!isExtension) {
            console.warn(errorMessage + err);
        }
        return;
    }

    if (
        !isExtension &&
        elementDocument.documentElement.dataset.ruffleOptout !== undefined
    ) {
        // Don't polyfill elements with the "data-ruffle-optout" attribute.
        return;
    }

    if (!isExtension) {
        if (!elementWindow.RufflePlayer) {
            const script = elementDocument.createElement("script");
            script.setAttribute("src", jsScriptUrl);
            script.onload = () => {
                // Inject parent configuration once the script is loaded, preventing it from being ignored.
                elementWindow.RufflePlayer = {};
                elementWindow.RufflePlayer.config = globalConfig;
            };
            elementDocument.head.appendChild(script);
        }
    } else {
        if (!elementWindow.RufflePlayer) {
            elementWindow.RufflePlayer = {};
        }
        // Merge parent window and frame configurations, will likely be applied too late though.
        elementWindow.RufflePlayer.config = {
            ...globalConfig,
            ...(elementWindow.RufflePlayer.config ?? {}),
        };
    }
}

/**
 * Listen for changes to the DOM.
 *
 */
function initMutationObserver(): void {
    const observer = new MutationObserver(function (mutationsList) {
        // If any nodes were added, re-run the polyfill to detect any new instances.
        const nodesAdded = mutationsList.some(
            (mutation) => mutation.addedNodes.length > 0
        );
        if (nodesAdded) {
            polyfillFlashInstances();
            polyfillFrames();
        }
    });
    observer.observe(document, { childList: true, subtree: true });
}

/**
 * Polyfills the detection of Flash plugins in the browser.
 */
export function pluginPolyfill(): void {
    installPlugin(FLASH_PLUGIN);
}

/**
 * Polyfills legacy Flash content on the page.
 *
 * @param isExt Whether or not Ruffle is running as a browser's extension.
 */
export function polyfill(isExt: boolean): void {
    isExtension = isExt;
    polyfillFlashInstances();
    polyfillFrames();
    initMutationObserver();
}
