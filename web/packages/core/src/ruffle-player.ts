import { Ruffle } from "../pkg/ruffle_web";

import { load_ruffle } from "./load-ruffle";
import { ruffleShadowTemplate } from "./shadow-template";
import { lookupElement } from "./register-element";

export const FLASH_MIMETYPE = "application/x-shockwave-flash";
export const FUTURESPLASH_MIMETYPE = "application/futuresplash";
export const FLASH7_AND_8_MIMETYPE = "application/x-shockwave-flash2-preview";
export const FLASH_MOVIE_MIMETYPE = "application/vnd.adobe.flash-movie";
export const FLASH_ACTIVEX_CLASSID =
    "clsid:D27CDB6E-AE6D-11cf-96B8-444553540000";

const DIMENSION_REGEX = /^\s*(\d+(\.\d+)?(%)?)/;

declare global {
    interface Document {
        webkitFullscreenEnabled?: boolean;
        webkitFullscreenElement?: HTMLElement;
        webkitCancelFullScreen?: () => void;
    }

    interface HTMLElement {
        webkitRequestFullScreen?: () => void;
    }
}

/**
 * An item to show in Ruffle's custom context menu
 */
interface ContextMenuItem {
    /**
     * The text to show to the user
     */
    text: string;

    /**
     * The function to call when clicked
     *
     * @param event The mouse event that triggered the click
     */
    onClick: (event: MouseEvent) => void;
}

function sanitizeParameters(
    parameters:
        | (URLSearchParams | string | Record<string, string>)
        | undefined
        | null
): Record<string, string> {
    if (parameters === null || parameters === undefined) {
        return {};
    }
    if (!(parameters instanceof URLSearchParams)) {
        parameters = new URLSearchParams(parameters);
    }
    const output: Record<string, string> = {};

    for (const [key, value] of parameters) {
        // Every value must be type of string
        output[key] = value.toString();
    }

    return output;
}

/**
 * The ruffle player element that should be inserted onto the page.
 *
 * This element will represent the rendered and intractable flash movie.
 */
export class RufflePlayer extends HTMLElement {
    private shadow: ShadowRoot;
    private dynamicStyles: HTMLStyleElement;
    private container: HTMLElement;
    private playButton: HTMLElement;
    private rightClickMenu: HTMLElement;
    private instance: Ruffle | null;
    private _trace_observer: ((message: string) => void) | null;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    private Ruffle: Promise<{ new (...args: any[]): Ruffle }>;
    private panicked = false;

    /**
     * If set to true, the movie is allowed to interact with the page through
     * JavaScript, using a flash concept called `ExternalInterface`.
     *
     * This should only be enabled for movies you trust.
     */
    allowScriptAccess: boolean;

    /**
     * Constructs a new Ruffle flash player for insertion onto the page.
     */
    constructor() {
        super();

        this.shadow = this.attachShadow({ mode: "closed" });
        this.shadow.appendChild(ruffleShadowTemplate.content.cloneNode(true));

        this.dynamicStyles = <HTMLStyleElement>(
            this.shadow.getElementById("dynamic_styles")
        );
        this.container = this.shadow.getElementById("container")!;
        this.playButton = this.shadow.getElementById("play_button")!;
        if (this.playButton) {
            this.playButton.addEventListener(
                "click",
                this.playButtonClicked.bind(this)
            );
        }
        this.rightClickMenu = this.shadow.getElementById("right_click_menu")!;

        this.addEventListener(
            "contextmenu",
            this.openRightClickMenu.bind(this)
        );

        window.addEventListener("click", this.hideRightClickMenu.bind(this));

        this.instance = null;
        this.allowScriptAccess = false;
        this._trace_observer = null;

        this.Ruffle = load_ruffle();

        return this;
    }

    /**
     * @ignore
     */
    connectedCallback(): void {
        this.updateStyles();
    }

    /**
     * @ignore
     */
    static get observedAttributes(): string[] {
        return ["width", "height"];
    }

    /**
     * @ignore
     */
    attributeChangedCallback(
        name: string,
        _oldValue: string | undefined,
        _newValue: string | undefined
    ): void {
        if (name === "width" || name === "height") {
            this.updateStyles();
        }
    }

    /**
     * @ignore
     */
    disconnectedCallback(): void {
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
            console.log("Ruffle instance destroyed.");
        }
    }

    /**
     * Updates the internal shadow DOM to reflect any set attributes from
     * this element.
     *
     * @protected
     */
    protected updateStyles(): void {
        if (this.dynamicStyles.sheet) {
            if (this.dynamicStyles.sheet.rules) {
                for (
                    let i = 0;
                    i < this.dynamicStyles.sheet.rules.length;
                    i++
                ) {
                    this.dynamicStyles.sheet.deleteRule(i);
                }
            }

            const widthAttr = this.attributes.getNamedItem("width");
            if (widthAttr !== undefined && widthAttr !== null) {
                const width = RufflePlayer.htmlDimensionToCssDimension(
                    widthAttr.value
                );
                if (width !== null) {
                    this.dynamicStyles.sheet.insertRule(
                        `:host { width: ${width}; }`
                    );
                }
            }

            const heightAttr = this.attributes.getNamedItem("height");
            if (heightAttr !== undefined && heightAttr !== null) {
                const height = RufflePlayer.htmlDimensionToCssDimension(
                    heightAttr.value
                );
                if (height !== null) {
                    this.dynamicStyles.sheet.insertRule(
                        `:host { height: ${height}; }`
                    );
                }
            }
        }
    }

    /**
     * Determine if this element is the fallback content of another Ruffle
     * player.
     *
     * This heuristic assumes Ruffle objects will never use their fallback
     * content. If this changes, then this code also needs to change.
     *
     * @private
     */
    private isUnusedFallbackObject(): boolean {
        let parent = this.parentNode;
        const element = lookupElement("ruffle-object");

        if (element !== null) {
            while (parent != document && parent != null) {
                if (parent.nodeName === element.name) {
                    return true;
                }

                parent = parent.parentNode;
            }
        }

        return false;
    }

    /**
     * Ensure a fresh Ruffle instance is ready on this player before continuing.
     *
     * @throws Any exceptions generated by loading Ruffle Core will be logged
     * and passed on.
     *
     * @private
     */
    private async ensureFreshInstance(): Promise<void> {
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
            console.log("Ruffle instance destroyed.");
        }

        const Ruffle = await this.Ruffle.catch((e) => {
            console.error("Serious error loading Ruffle: " + e);

            // Serious duck typing. In error conditions, let's not make assumptions.
            const message =
                e && e.message ? String(e.message).toLowerCase() : "";
            if (message.indexOf("mime") >= 0) {
                this.panicked = true;
                this.container.innerHTML = `
                    <div id="panic">
                        <div id="panic-title">Something went wrong :(</div>
                        <div id="panic-body">
                            <p>Ruffle has encountered a major issue whilst trying to initialize.</p>
                            <p>This web server is either not serving ".wasm" files with the correct MIME type, or the file cannot be found.</p>
                            <p>If you are the server administrator, please consult the Ruffle wiki for help.</p>
                        </div>
                        <div id="panic-footer">
                            <ul>
                                <li><a href="https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#configure-wasm-mime-type">view Ruffle wiki</a></li>
                            </ul>
                        </div>
                    </div>
                `;
            }
            throw e;
        });

        this.instance = new Ruffle(
            this.container,
            this,
            this.allowScriptAccess
        );
        console.log("New Ruffle instance created.");
    }

    /**
     * Load a movie into this Ruffle Player instance by URL.
     *
     * Any existing movie will be immediately stopped, while the new movie's
     * load happens asynchronously. There is currently no way to await the file
     * being loaded, or any errors that happen loading it.
     *
     * @param url The URL to stream.
     * @param parameters The parameters (also known as "flashvars") to load the movie with.
     * If it's a string, it will be decoded into an object.
     * If it's an object, every key and value must be a String.
     * These parameters will be merged onto any found in the query portion of the swf URL.
     */
    async streamSwfUrl(
        url: string,
        parameters:
            | URLSearchParams
            | string
            | Record<string, string>
            | undefined
            | null
    ): Promise<void> {
        //TODO: Actually stream files...
        try {
            if (this.isConnected && !this.isUnusedFallbackObject()) {
                console.log("Loading SWF file " + url);

                await this.ensureFreshInstance();
                parameters = {
                    ...sanitizeParameters(url.substring(url.indexOf("?"))),
                    ...sanitizeParameters(parameters),
                };
                this.instance!.stream_from(url, parameters);

                if (this.playButton) {
                    this.playButton.style.display = "block";
                }
            } else {
                console.warn(
                    "Ignoring attempt to play a disconnected or suspended Ruffle element"
                );
            }
        } catch (err) {
            console.error("Serious error occurred loading SWF file: " + err);
            this.panic(err);
            throw err;
        }
    }

    private playButtonClicked(): void {
        if (this.instance) {
            this.instance.play();
            if (this.playButton) {
                this.playButton.style.display = "none";
            }
        }
    }

    /**
     * Checks if this player is allowed to be fullscreen by the browser.
     *
     * @returns True if you may call [[enterFullscreen]].
     */
    get fullscreenEnabled(): boolean {
        return !!(
            document.fullscreenEnabled || document.webkitFullscreenEnabled
        );
    }

    /**
     * Checks if this player is currently fullscreen inside the browser.
     *
     * @returns True if it is fullscreen.
     */
    get isFullscreen(): boolean {
        return (
            (document.fullscreenElement || document.webkitFullscreenElement) ===
            this
        );
    }

    /**
     * Requests the browser to make this player fullscreen.
     *
     * This is not guaranteed to succeed, please check [[fullscreenEnabled]] first.
     */
    enterFullscreen(): void {
        if (this.requestFullscreen) {
            this.requestFullscreen();
        } else if (this.webkitRequestFullScreen) {
            this.webkitRequestFullScreen();
        }
    }

    /**
     * Requests the browser to no longer make this player fullscreen.
     */
    exitFullscreen(): void {
        if (document.exitFullscreen) {
            document.exitFullscreen();
        } else if (document.webkitCancelFullScreen) {
            document.webkitCancelFullScreen();
        }
    }

    private contextMenuItems(): ContextMenuItem[] {
        const items = [];
        if (this.fullscreenEnabled) {
            if (this.isFullscreen) {
                items.push({
                    text: "Exit fullscreen",
                    onClick: this.exitFullscreen.bind(this),
                });
            } else {
                items.push({
                    text: "Enter fullscreen",
                    onClick: this.enterFullscreen.bind(this),
                });
            }
        }
        items.push({
            // TODO: ruffle version info
            text: `Ruffle ${
                __CHANNEL__ === "nightly"
                    ? `nightly ${__COMMIT_DATE__}`
                    : "0.1.0"
            }`,
            onClick() {
                window.open("https://ruffle.rs/", "_blank");
            },
        });
        return items;
    }

    private openRightClickMenu(e: MouseEvent): void {
        e.preventDefault();

        // Clear all `right_click_menu` items.
        while (this.rightClickMenu.firstChild) {
            this.rightClickMenu.removeChild(this.rightClickMenu.firstChild);
        }

        // Populate `right_click_menu` items.
        for (const { text, onClick } of this.contextMenuItems()) {
            const element = document.createElement("li");
            element.className = "menu_item active";
            element.textContent = text;
            element.addEventListener("click", onClick);
            this.rightClickMenu.appendChild(element);
        }

        // Place `right_click_menu` in the top-left corner, so
        // its `clientWidth` and `clientHeight` are not clamped.
        this.rightClickMenu.style.left = "0";
        this.rightClickMenu.style.top = "0";
        this.rightClickMenu.style.display = "block";

        const rect = this.getBoundingClientRect();
        const x = e.clientX - rect.x;
        const y = e.clientY - rect.y;
        const maxX = rect.width - this.rightClickMenu.clientWidth - 1;
        const maxY = rect.height - this.rightClickMenu.clientHeight - 1;

        this.rightClickMenu.style.left = Math.floor(Math.min(x, maxX)) + "px";
        this.rightClickMenu.style.top = Math.floor(Math.min(y, maxY)) + "px";
    }

    private hideRightClickMenu(): void {
        this.rightClickMenu.style.display = "none";
    }

    /**
     * Pauses this player.
     *
     * No more frames, scripts or sounds will be executed.
     * This movie will be considered inactive and will not wake up until resumed.
     */
    pause(): void {
        if (this.instance) {
            this.instance.pause();
            if (this.playButton) {
                this.playButton.style.display = "block";
            }
        }
    }

    /**
     * Load a movie's data into this Ruffle Player instance.
     *
     * Any existing movie will be immediately stopped, and the new movie's data
     * placed into a fresh Stage on the same stack.
     *
     * Please note that by doing this, no URL information will be provided to
     * the movie being loaded.
     *
     * @param data The data to stream.
     * @param parameters The parameters (also known as "flashvars") to load the movie with.
     * If it's a string, it will be decoded into an object.
     * If it's an object, every key and value must be a String.
     */
    async playSwfData(
        data: Iterable<number>,
        parameters:
            | URLSearchParams
            | string
            | Record<string, string>
            | undefined
            | null
    ): Promise<void> {
        try {
            if (this.isConnected && !this.isUnusedFallbackObject()) {
                console.log("Got SWF data");

                await this.ensureFreshInstance();
                this.instance?.load_data(
                    new Uint8Array(data),
                    sanitizeParameters(parameters)
                );
                console.log("New Ruffle instance created.");

                if (this.playButton) {
                    this.playButton.style.display = "block";
                }
            } else {
                console.warn(
                    "Ignoring attempt to play a disconnected or suspended Ruffle element"
                );
            }
        } catch (err) {
            console.error("Serious error occurred loading SWF file: " + err);
            this.panic(err);
            throw err;
        }
    }

    /**
     * Copies attributes and children from another element to this player element.
     * Used by the polyfill elements, RuffleObject and RuffleEmbed.
     *
     * @param elem The element to copy all attributes from.
     *
     * @protected
     */
    protected copyElement(elem: HTMLElement): void {
        if (elem) {
            for (let i = 0; i < elem.attributes.length; i++) {
                const attrib = elem.attributes[i];
                if (attrib.specified) {
                    // Issue 468: Chrome "Click to Active Flash" box stomps on title attribute
                    if (
                        attrib.name === "title" &&
                        attrib.value === "Adobe Flash Player"
                    ) {
                        continue;
                    }

                    try {
                        this.setAttribute(attrib.name, attrib.value);
                    } catch (err) {
                        // The embed may have invalid attributes, so handle these gracefully.
                        console.warn(
                            `Unable to set attribute ${attrib.name} on Ruffle instance`
                        );
                    }
                }
            }

            for (const node of Array.from(elem.children)) {
                this.appendChild(node);
            }
        }
    }

    /**
     * Converts a dimension attribute on an HTML embed/object element to a valid CSS dimension.
     * HTML element dimensions are unitless, but can also be percentages.
     * Add a 'px' unit unless the value is a percentage.
     * Returns null if this is not a valid dimension.
     *
     * @param attribute The attribute to convert
     *
     * @private
     */
    private static htmlDimensionToCssDimension(
        attribute: string
    ): string | null {
        if (attribute) {
            const match = attribute.match(DIMENSION_REGEX);
            if (match) {
                let out = match[1];
                if (!match[3]) {
                    // Unitless -- add px for CSS.
                    out += "px";
                }
                return out;
            }
        }
        return null;
    }

    /**
     * When a movie presents a new callback through `ExternalInterface.addCallback`,
     * we are informed so that we can expose the method on any relevant DOM element.
     *
     * This should only be called by Ruffle itself and not by users.
     *
     * @param name The name of the callback that is now available.
     *
     * @internal
     * @ignore
     */
    onCallbackAvailable(name: string): void {
        const instance = this.instance;
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (<any>this)[name] = (...args: any[]) => {
            return instance?.call_exposed_callback(name, args);
        };
    }

    /**
     * Sets a trace observer on this flash player.
     *
     * The observer will be called, as a function, for each message that the playing movie will "trace" (output).
     *
     * @param observer The observer that will be called for each trace.
     */
    set traceObserver(observer: ((message: string) => void) | null) {
        this.instance?.set_trace_observer(observer);
    }

    /**
     * Panics this specific player, forcefully destroying all resources and displays an error message to the user.
     *
     * This should be called when something went absolutely, incredibly and disastrously wrong and there is no chance
     * of recovery.
     *
     * Ruffle will attempt to isolate all damage to this specific player instance, but no guarantees can be made if there
     * was a core issue which triggered the panic. If Ruffle is unable to isolate the cause to a specific player, then
     * all players will panic and Ruffle will become "poisoned" - no more players will run on this page until it is
     * reloaded fresh.
     *
     * @param error The error, if any, that triggered this panic.
     */
    panic(error: Error | null): void {
        if (this.panicked) {
            // Only show the first major error, not any repeats - they aren't as important
            return;
        }
        this.panicked = true;

        // Clears out any existing content (ie play button or canvas) and replaces it with the error screen
        this.container.innerHTML = `
            <div id="panic">
                <div id="panic-title">Something went wrong :(</div>
                <div id="panic-body">
                    <p>Ruffle has encountered a major issue whilst trying to display this Flash content.</p>
                    <p>This isn't supposed to happen, so we'd really appreciate if you could file a bug!</p>
                </div>
                <div id="panic-footer">
                    <ul>
                        <li><a href="https://github.com/ruffle-rs/ruffle/issues/new">report bug</a></li>
                        <li><a href="#" id="panic-view-details">view error details</a></li>
                    </ul>
                </div>
            </div>
        `;
        (<HTMLLinkElement>(
            this.container.querySelector("#panic-view-details")
        )).onclick = () => {
            let errorText = "# Error Info\n";

            if (error instanceof Error) {
                errorText += `Error name: ${error.name}\n`;
                errorText += `Error message: ${error.message}\n`;
                if (error.stack) {
                    errorText += `Error stack:\n\`\`\`\n${error.stack}\n\`\`\`\n`;
                }
            } else {
                errorText += `Error: ${error}\n`;
            }

            errorText += "\n# Player Info\n";
            errorText += this.debugPlayerInfo();

            errorText += "\n# Page Info\n";
            errorText += `Page URL: ${document.location.href}\n`;

            errorText += "\n# Browser Info\n";
            errorText += `Useragent: ${window.navigator.userAgent}\n`;
            errorText += `OS: ${window.navigator.platform}\n`;

            // TODO: Ruffle source version. No way to know right now?

            this.container.querySelector(
                "#panic-body"
            )!.innerHTML = `<textarea>${errorText}</textarea>`;
            return false;
        };

        // Do this last, just in case it causes any cascading issues.
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
        }
    }

    protected debugPlayerInfo(): string {
        return `Allows script access: ${this.allowScriptAccess}\n`;
    }
}

/**
 * Returns whether the given filename ends in a known flash extension.
 *
 * @param filename The filename to test.
 * @return True if the filename is a flash movie (swf or spl).
 */
export function isSwfFilename(filename: string | null): boolean {
    return !!(
        filename &&
        (filename.search(/\.swf(?:[?#]|$)/i) >= 0 ||
            filename.search(/\.spl(?:[?#]|$)/i) >= 0)
    );
}
