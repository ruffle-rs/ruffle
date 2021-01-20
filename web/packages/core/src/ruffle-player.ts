import { Ruffle } from "../pkg/ruffle_web";

import { loadRuffle } from "./load-ruffle";
import { ruffleShadowTemplate } from "./shadow-template";
import { lookupElement } from "./register-element";
import { Config } from "./config";
import {
    BaseLoadOptions,
    DataLoadOptions,
    URLLoadOptions,
    AutoPlay,
    UnmuteOverlay,
} from "./load-options";

export const FLASH_MIMETYPE = "application/x-shockwave-flash";
export const FUTURESPLASH_MIMETYPE = "application/futuresplash";
export const FLASH7_AND_8_MIMETYPE = "application/x-shockwave-flash2-preview";
export const FLASH_MOVIE_MIMETYPE = "application/vnd.adobe.flash-movie";
export const FLASH_ACTIVEX_CLASSID =
    "clsid:D27CDB6E-AE6D-11cf-96B8-444553540000";

const RUFFLE_ORIGIN = "https://ruffle.rs";
const DIMENSION_REGEX = /^\s*(\d+(\.\d+)?(%)?)/;

enum PanicError {
    Unknown,
    FileProtocol,
    JavascriptConfiguration,
    JavascriptConflict,
    WasmCors,
    WasmMimeType,
    WasmNotFound,
}

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

/**
 * Converts arbitrary input to an easy to use record object.
 *
 * @param parameters Parameters to sanitize
 * @returns A sanitized map of param name to param value
 */
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
    private unmuteOverlay: HTMLElement;
    private rightClickMenu: HTMLElement;
    private swfUrl?: string;
    private instance: Ruffle | null;
    private _trace_observer: ((message: string) => void) | null;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    private ruffleConstructor: Promise<{ new (...args: any[]): Ruffle }>;
    private panicked = false;

    /**
     * If set to true, the movie is allowed to interact with the page through
     * JavaScript, using a flash concept called `ExternalInterface`.
     *
     * This should only be enabled for movies you trust.
     */
    allowScriptAccess: boolean;

    /**
     * Any configuration that should apply to this specific player.
     * This will be defaulted with any global configuration.
     */
    config: Config = {};

    /**
     * Constructs a new Ruffle flash player for insertion onto the page.
     */
    constructor() {
        super();

        this.shadow = this.attachShadow({ mode: "open" });
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

        this.unmuteOverlay = this.shadow.getElementById("unmute_overlay")!;
        this.unmuteOverlay.addEventListener(
            "click",
            this.unmuteOverlayClicked.bind(this)
        );

        this.rightClickMenu = this.shadow.getElementById("right_click_menu")!;

        this.addEventListener(
            "contextmenu",
            this.openRightClickMenu.bind(this)
        );

        window.addEventListener("click", this.hideRightClickMenu.bind(this));

        this.instance = null;
        this.allowScriptAccess = false;
        this._trace_observer = null;

        this.ruffleConstructor = loadRuffle();

        return this;
    }

    /**
     * @ignore
     * @internal
     */
    connectedCallback(): void {
        this.updateStyles();
    }

    /**
     * @ignore
     * @internal
     */
    static get observedAttributes(): string[] {
        return ["width", "height"];
    }

    /**
     * @ignore
     * @internal
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
     * @internal
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
    private async ensureFreshInstance(config: BaseLoadOptions): Promise<void> {
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
            console.log("Ruffle instance destroyed.");
        }

        const ruffleConstructor = await this.ruffleConstructor.catch((e) => {
            console.error(`Serious error loading Ruffle: ${e}`);

            // Serious duck typing. In error conditions, let's not make assumptions.
            if (window.location.protocol === "file:") {
                e.ruffleIndexError = PanicError.FileProtocol;
            } else if (!e.ruffleIsExtension) {
                e.ruffleIndexError = PanicError.WasmNotFound;
                const message = String(e.message).toLowerCase();
                if (message.includes("mime")) {
                    e.ruffleIndexError = PanicError.WasmMimeType;
                } else if (
                    message.includes("networkerror") ||
                    message.includes("failed to fetch")
                ) {
                    e.ruffleIndexError = PanicError.WasmCors;
                } else if (
                    !message.includes("magic") &&
                    (e.name === "CompileError" || e.name === "TypeError")
                ) {
                    e.ruffleIndexError = PanicError.JavascriptConflict;
                }
            }
            this.panic(e);
            throw e;
        });

        this.instance = new ruffleConstructor(
            this.container,
            this,
            this.allowScriptAccess,
            config
        );
        console.log("New Ruffle instance created.");

        // In Firefox, AudioContext.state is always "suspended" when the object has just been created.
        // It may change by itself to "running" some milliseconds later. So we need to wait a little
        // bit before checking if autoplay is supported and applying the instance config.
        if (this.audioState() !== "running") {
            this.container.style.visibility = "hidden";
            await new Promise<void>((resolve) => {
                window.setTimeout(() => {
                    resolve();
                }, 200);
            });
            this.container.style.visibility = "";
        }

        const autoplay = Object.values(Object(AutoPlay)).includes(
            config.autoplay
        )
            ? config.autoplay
            : AutoPlay.Auto;
        const unmuteVisibility = Object.values(Object(UnmuteOverlay)).includes(
            config.unmuteOverlay
        )
            ? config.unmuteOverlay
            : UnmuteOverlay.Visible;

        if (
            autoplay == AutoPlay.On ||
            (autoplay == AutoPlay.Auto && this.audioState() === "running")
        ) {
            this.play();

            if (this.audioState() !== "running") {
                this.unmuteOverlay.style.display = "block";

                // We need to mark each child as hidden or visible, as we want an overlay even if it's "hidden".
                // We need to undo this later if the config changed back to visible, but we already hid them.
                this.unmuteOverlay.childNodes.forEach((node) => {
                    if ("style" in node) {
                        const style = (<ElementCSSInlineStyle>node).style;
                        style.visibility =
                            unmuteVisibility == UnmuteOverlay.Visible
                                ? ""
                                : "hidden";
                    }
                });

                const audioContext = this.instance?.audio_context();
                if (audioContext) {
                    audioContext.onstatechange = () => {
                        if (audioContext.state === "running") {
                            this.unmuteOverlayClicked();
                        }
                        audioContext.onstatechange = null;
                    };
                }
            }
        } else {
            this.playButton.style.display = "block";
        }
    }

    /**
     * Loads a specified movie into this player.
     *
     * This will replace any existing movie that may be playing.
     *
     * @param options One of the following:
     * - A URL, passed as a string, which will load a URL with default options.
     * - A [[URLLoadOptions]] object, to load a URL with options.
     * - A [[DataLoadOptions]] object, to load data with options.
     *
     * The options will be defaulted by the [[config]] field, which itself
     * is defaulted by a global `window.RufflePlayer.config`.
     */
    async load(
        options: string | URLLoadOptions | DataLoadOptions
    ): Promise<void> {
        let optionsError = "";
        switch (typeof options) {
            case "string":
                options = { url: options };
                break;
            case "object":
                if (options === null) {
                    optionsError = "Argument 0 must be a string or object";
                } else if (!("url" in options) && !("data" in options)) {
                    optionsError =
                        "Argument 0 must contain a `url` or `data` key";
                } else if (
                    "url" in options &&
                    typeof options.url !== "string"
                ) {
                    optionsError = "`url` must be a string";
                }
                break;
            default:
                optionsError = "Argument 0 must be a string or object";
                break;
        }
        if (optionsError.length > 0) {
            const error = new TypeError(optionsError);
            error.ruffleIndexError = PanicError.JavascriptConfiguration;
            this.panic(error);
            throw error;
        }

        //TODO: Actually stream files...
        try {
            if (this.isConnected && !this.isUnusedFallbackObject()) {
                const config: BaseLoadOptions = {
                    ...(window.RufflePlayer?.config ?? {}),
                    ...this.config,
                    ...options,
                };

                // Pre-emptively set background color of container while Ruffle/SWF loads.
                if (config.backgroundColor) {
                    this.container.style.backgroundColor =
                        config.backgroundColor;
                }

                await this.ensureFreshInstance(config);

                if ("url" in options) {
                    console.log(`Loading SWF file ${options.url}`);
                    try {
                        this.swfUrl = new URL(
                            options.url,
                            document.location.href
                        ).href;
                    } catch {
                        this.swfUrl = options.url;
                    }

                    const parameters = {
                        ...sanitizeParameters(
                            options.url.substring(options.url.indexOf("?"))
                        ),
                        ...sanitizeParameters(options.parameters),
                    };
                    this.instance!.stream_from(options.url, parameters);
                } else if ("data" in options) {
                    console.log("Loading SWF data");
                    this.instance!.load_data(
                        new Uint8Array(options.data),
                        sanitizeParameters(options.parameters)
                    );
                }
            } else {
                console.warn(
                    "Ignoring attempt to play a disconnected or suspended Ruffle element"
                );
            }
        } catch (err) {
            console.error(`Serious error occurred loading SWF file: ${err}`);
            throw err;
        }
    }

    private playButtonClicked(): void {
        this.play();
    }

    /**
     * Plays or resumes the movie.
     */
    play(): void {
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
            text: `Ruffle %VERSION_NAME%`,
            onClick() {
                window.open(RUFFLE_ORIGIN, "_blank");
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

    private audioState(): string {
        if (this.instance) {
            const audioContext = this.instance.audio_context();
            return (audioContext && audioContext.state) || "running";
        }
        return "suspended";
    }

    private unmuteOverlayClicked(): void {
        if (this.instance) {
            if (this.audioState() !== "running") {
                const audioContext = this.instance.audio_context();
                if (audioContext) {
                    audioContext.resume();
                }
            }
            if (this.unmuteOverlay) {
                this.unmuteOverlay.style.display = "none";
            }
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

        if (
            error instanceof Error &&
            (error.name === "AbortError" ||
                error.message.includes("AbortError"))
        ) {
            // Firefox: Don't display the panic screen if the user leaves the page while something is still loading
            return;
        }

        const errorIndex = error?.ruffleIndexError ?? PanicError.Unknown;

        const errorArray: Array<string | null> & {
            stackIndex: number;
        } = Object.assign([], {
            stackIndex: -1,
        });

        errorArray.push("# Error Info\n");

        if (error instanceof Error) {
            errorArray.push(`Error name: ${error.name}\n`);
            errorArray.push(`Error message: ${error.message}\n`);
            if (error.stack) {
                const stackIndex =
                    errorArray.push(
                        `Error stack:\n\`\`\`\n${error.stack}\n\`\`\`\n`
                    ) - 1;
                errorArray.stackIndex = stackIndex;
            }
        } else {
            errorArray.push(`Error: ${error}\n`);
        }

        errorArray.push("\n# Player Info\n");
        errorArray.push(this.debugPlayerInfo());

        errorArray.push("\n# Page Info\n");
        errorArray.push(`Page URL: ${document.location.href}\n`);
        if (this.swfUrl) errorArray.push(`SWF URL: ${this.swfUrl}\n`);

        errorArray.push("\n# Browser Info\n");
        errorArray.push(`Useragent: ${window.navigator.userAgent}\n`);
        errorArray.push(`OS: ${window.navigator.platform}\n`);

        errorArray.push("\n# Ruffle Info\n");
        errorArray.push(`Version: %VERSION_NUMBER%\n`);
        errorArray.push(`Name: %VERSION_NAME%\n`);
        errorArray.push(`Channel: %VERSION_CHANNEL%\n`);
        errorArray.push(`Built: %BUILD_DATE%\n`);
        errorArray.push(`Commit: %COMMIT_HASH%\n`);

        const errorText = errorArray.join("");

        const issueTitle = `Error on ${document.location.href}`;
        let issueLink = `https://github.com/ruffle-rs/ruffle/issues/new?title=${encodeURIComponent(
            issueTitle
        )}&body=`;
        let issueBody = encodeURIComponent(errorText);
        if (
            errorArray.stackIndex > -1 &&
            String(issueLink + issueBody).length > 8195
        ) {
            // Strip the stack error from the array when the produced URL is way too long.
            // This should prevent "414 Request-URI Too Large" errors on Github.
            errorArray[errorArray.stackIndex] = null;
            issueBody = encodeURIComponent(errorArray.join(""));
        }
        issueLink += issueBody;

        // Clears out any existing content (ie play button or canvas) and replaces it with the error screen
        let errorBody, errorFooter;
        switch (errorIndex) {
            case PanicError.FileProtocol:
                // General error: Running on the `file:` protocol
                errorBody = `
                    <p>It appears you are running Ruffle on the "file:" protocol.</p>
                    <p>This doesn't work as browsers block many features from working for security reasons.</p>
                    <p>Instead, we invite you to setup a local server or either use the web demo or the desktop application.</p>
                `;
                errorFooter = `
                    <li><a target="_top" href="${RUFFLE_ORIGIN}/demo">Web Demo</a></li>
                    <li><a target="_top" href="https://github.com/ruffle-rs/ruffle/tags">Desktop Application</a></li>
                `;
                break;
            case PanicError.JavascriptConfiguration:
                // General error: Incorrect JavaScript configuration
                errorBody = `
                    <p>Ruffle has encountered a major issue due to an incorrect JavaScript configuration.</p>
                    <p>If you are the server administrator, we invite you to check the error details to find out which parameter is at fault.</p>
                    <p>You can also consult the Ruffle wiki for help.</p>
                `;
                errorFooter = `
                    <li><a target="_top" href="https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#javascript-api">View Ruffle Wiki</a></li>
                    <li><a href="#" id="panic-view-details">View Error Details</a></li>
                `;
                break;
            case PanicError.WasmNotFound:
                // Self hosted: Cannot load `.wasm` file - file not found
                errorBody = `
                    <p>Ruffle failed to load the required ".wasm" file component.</p>
                    <p>If you are the server administrator, please ensure the file has correctly been uploaded.</p>
                    <p>If the issue persists, you may need to use the "publicPath" setting: please consult the Ruffle wiki for help.</p>
                `;
                errorFooter = `
                    <li><a target="_top" href="https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#configuration-options">View Ruffle Wiki</a></li>
                    <li><a href="#" id="panic-view-details">View Error Details</a></li>
                `;
                break;
            case PanicError.WasmMimeType:
                // Self hosted: Cannot load `.wasm` file - incorrect MIME type
                errorBody = `
                    <p>Ruffle has encountered a major issue whilst trying to initialize.</p>
                    <p>This web server is not serving ".wasm" files with the correct MIME type.</p>
                    <p>If you are the server administrator, please consult the Ruffle wiki for help.</p>
                `;
                errorFooter = `
                    <li><a target="_top" href="https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#configure-webassembly-mime-type">View Ruffle Wiki</a></li>
                    <li><a href="#" id="panic-view-details">View Error Details</a></li>
                `;
                break;
            case PanicError.WasmCors:
                // Self hosted: Cannot load `.wasm` file - CORS issues
                errorBody = `
                    <p>Ruffle failed to load the required ".wasm" file component.</p>
                    <p>Access to fetch has likely been blocked by CORS policy.</p>
                    <p>If you are the server administrator, please consult the Ruffle wiki for help.</p>
                `;
                errorFooter = `
                    <li><a target="_top" href="https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#web">View Ruffle Wiki</a></li>
                    <li><a href="#" id="panic-view-details">View Error Details</a></li>
                `;
                break;
            case PanicError.JavascriptConflict:
                // Self hosted: Cannot load `.wasm` file - a native object / function is overriden
                errorBody = `
                    <p>Ruffle has encountered a major issue whilst trying to initialize.</p>
                    <p>It seems like this page uses JavaScript code that conflicts with Ruffle.</p>
                    <p>If you are the server administrator, we invite you to try loading the file on a blank page.</p>
                `;
                errorFooter = `
                    <li><a target="_top" href="${issueLink}">Report Bug</a></li>
                    <li><a href="#" id="panic-view-details">View Error Details</a></li>
                `;
                break;
            default:
                // Unknown error
                errorBody = `
                    <p>Ruffle has encountered a major issue whilst trying to display this Flash content.</p>
                    <p>This isn't supposed to happen, so we'd really appreciate if you could file a bug!</p>
                `;
                errorFooter = `
                    <li><a target="_top" href="${issueLink}">Report Bug</a></li>
                    <li><a href="#" id="panic-view-details">View Error Details</a></li>
                `;
                break;
        }
        this.container.innerHTML = `
            <div id="panic">
                <div id="panic-title">Something went wrong :(</div>
                <div id="panic-body">${errorBody}</div>
                <div id="panic-footer">
                    <ul>${errorFooter}</ul>
                </div>
            </div>
        `;
        const viewDetails = <HTMLLinkElement>(
            this.container.querySelector("#panic-view-details")
        );
        if (viewDetails) {
            viewDetails.onclick = () => {
                this.container.querySelector(
                    "#panic-body"
                )!.innerHTML = `<textarea>${errorText}</textarea>`;
                return false;
            };
        }

        // Do this last, just in case it causes any cascading issues.
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
        }
    }

    displayMessage(message: string): void {
        // Show a dismissible message in front of the player
        const div = document.createElement("div");
        div.id = "message_overlay";
        div.innerHTML = `<div class="message">
            <div>
                <p>${message}</p>
            </div>
            <div>
                <button id="continue-btn">continue</button>
            </div>`;
        this.container.prepend(div);
        (<HTMLButtonElement>(
            this.container.querySelector("#continue-btn")
        )).onclick = () => {
            div.remove();
        };
    }

    protected debugPlayerInfo(): string {
        return `Allows script access: ${this.allowScriptAccess}\n`;
    }
}

/**
 * Returns whether a SWF file can call JavaScript code in the surrounding HTML file.
 *
 * @param access The value of the `allowScriptAccess` attribute.
 * @param url The URL of the SWF file.
 * @returns True if script access is allowed.
 */
export function isScriptAccessAllowed(
    access: string | null,
    url: string
): boolean {
    if (!access) {
        access = "sameDomain";
    }
    switch (access.toLowerCase()) {
        case "always":
            return true;
        case "never":
            return false;
        case "samedomain":
        default:
            try {
                return (
                    new URL(window.location.href).origin ===
                    new URL(url, window.location.href).origin
                );
            } catch {
                return false;
            }
    }
}

/**
 * Returns whether the given filename ends in a known flash extension.
 *
 * @param filename The filename to test.
 * @returns True if the filename is a flash movie (swf or spl).
 */
export function isSwfFilename(filename: string | null): boolean {
    if (filename) {
        let pathname = "";
        try {
            // A base URL is required if `filename` is a relative URL, but we don't need to detect the real URL origin.
            pathname = new URL(filename, RUFFLE_ORIGIN).pathname;
        } catch (err) {
            // Some invalid filenames, like `///`, could raise a TypeError. Let's fail silently in this situation.
        }
        if (pathname && pathname.length >= 4) {
            const extension = pathname.slice(-4).toLowerCase();
            if (extension === ".swf" || extension === ".spl") {
                return true;
            }
        }
    }
    return false;
}
