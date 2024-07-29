import type { DataLoadOptions, URLLoadOptions } from "./load-options";
import type { MovieMetadata } from "./movie-metadata";
import { InnerPlayer, ReadyState } from "./internal/player/inner";

/**
 * The ruffle player element that should be inserted onto the page.
 *
 * This element will represent the rendered and intractable flash movie.
 */
export class RufflePlayer extends HTMLElement {
    #inner: InnerPlayer;

    get onFSCommand(): ((command: string, args: string) => boolean) | null {
        return this.#inner.onFSCommand;
    }

    set onFSCommand(
        value: ((command: string, args: string) => boolean) | null,
    ) {
        this.#inner.onFSCommand = value;
    }

    /**
     * Indicates the readiness of the playing movie.
     *
     * @returns The `ReadyState` of the player.
     */
    get readyState(): ReadyState {
        return this.#inner._readyState;
    }

    /**
     * The metadata of the playing movie (such as movie width and height).
     * These are inherent properties stored in the SWF file and are not affected by runtime changes.
     * For example, `metadata.width` is the width of the SWF file, and not the width of the Ruffle player.
     *
     * @returns The metadata of the movie, or `null` if the movie metadata has not yet loaded.
     */
    get metadata(): MovieMetadata | null {
        return this.#inner.metadata;
    }

    /**
     * Constructs a new Ruffle flash player for insertion onto the page.
     */
    constructor() {
        super();
        this.#inner = new InnerPlayer(this, () => this.debugPlayerInfo());
    }

    /**
     * @ignore
     * @internal
     */
    connectedCallback(): void {
        this.#inner.updateStyles();
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
        _newValue: string | undefined,
    ): void {
        if (name === "width" || name === "height") {
            this.#inner.updateStyles();
        }
    }

    /**
     * @ignore
     * @internal
     */
    disconnectedCallback(): void {
        this.#inner.destroy();
    }

    /**
     * Reloads the player, as if you called {@link RufflePlayer.load} with the same config as the last time it was called.
     *
     * If this player has never been loaded, this method will return an error.
     */
    async reload(): Promise<void> {
        await this.#inner.reload();
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
     * The options, if provided, must only contain values provided for this specific movie.
     * They must not contain any default values, since those would overwrite other configuration
     * settings with a lower priority (e.g. the general RufflePlayer config).
     * @param isPolyfillElement Whether the element is a polyfilled Flash element or not.
     * This is used to determine a default value of the configuration.
     *
     * The options will be defaulted by the [[config]] field, which itself
     * is defaulted by a global `window.RufflePlayer.config`.
     */
    async load(
        options: string | URLLoadOptions | DataLoadOptions,
        isPolyfillElement: boolean = false,
    ): Promise<void> {
        await this.#inner.load(options, isPolyfillElement);
    }

    /**
     * Plays or resumes the movie.
     */
    play(): void {
        this.#inner.play();
    }

    /**
     * Whether this player is currently playing.
     *
     * @returns True if this player is playing, false if it's paused or hasn't started yet.
     */
    get isPlaying(): boolean {
        return this.#inner.isPlaying;
    }

    /**
     * Returns the master volume of the player.
     *
     * The volume is linear and not adapted for logarithmic hearing.
     *
     * @returns The volume. 1.0 is 100% volume.
     */
    get volume(): number {
        return this.#inner.volume;
    }

    /**
     * Sets the master volume of the player.
     *
     * The volume should be linear and not adapted for logarithmic hearing.
     *
     * @param value The volume. 1.0 is 100% volume.
     */
    set volume(value: number) {
        this.#inner.volume = value;
    }

    /**
     * Checks if this player is allowed to be fullscreen by the browser.
     *
     * @returns True if you may call [[enterFullscreen]].
     */
    get fullscreenEnabled(): boolean {
        return this.#inner.fullscreenEnabled;
    }

    /**
     * Checks if this player is currently fullscreen inside the browser.
     *
     * @returns True if it is fullscreen.
     */
    get isFullscreen(): boolean {
        return this.#inner.isFullscreen;
    }

    /**
     * Exported function that requests the browser to change the fullscreen state if
     * it is allowed.
     *
     * @param isFull Whether to set to fullscreen or return to normal.
     */
    setFullscreen(isFull: boolean): void {
        this.#inner.setFullscreen(isFull);
    }

    /**
     * Requests the browser to make this player fullscreen.
     *
     * This is not guaranteed to succeed, please check [[fullscreenEnabled]] first.
     */
    enterFullscreen(): void {
        this.#inner.enterFullscreen();
    }

    /**
     * Requests the browser to no longer make this player fullscreen.
     */
    exitFullscreen(): void {
        this.#inner.exitFullscreen();
    }

    /**
     * Fetches the loaded SWF and downloads it.
     */
    async downloadSwf(): Promise<void> {
        await this.#inner.downloadSwf();
    }

    /**
     * Pauses this player.
     *
     * No more frames, scripts or sounds will be executed.
     * This movie will be considered inactive and will not wake up until resumed.
     */
    pause(): void {
        this.#inner.pause();
    }

    /**
     * Sets a trace observer on this flash player.
     *
     * The observer will be called, as a function, for each message that the playing movie will "trace" (output).
     *
     * @param observer The observer that will be called for each trace.
     */
    set traceObserver(observer: ((message: string) => void) | null) {
        this.#inner.traceObserver = observer;
    }

    protected debugPlayerInfo(): string {
        return "";
    }

    /** @ignore */
    public PercentLoaded(): number {
        // [NA] This is a stub - we need to research how this is actually implemented (is it just base swf loadedBytes?)
        if (this.readyState === ReadyState.Loaded) {
            return 100;
        } else {
            return 0;
        }
    }

    get config(): URLLoadOptions | DataLoadOptions | object {
        return this.#inner.config;
    }

    set config(value: URLLoadOptions | DataLoadOptions | object) {
        this.#inner.config = value;
    }
}

/**
 * Copies attributes and children from another element to a target element.
 * Used by the polyfill elements, RuffleObject and RuffleEmbed.
 *
 * @param element The element to copy all attributes from.
 * @param destination The element to copy all attributes to.
 */
export function copyElement(element: Element, destination: Element): void {
    if (element) {
        for (const attribute of element.attributes) {
            if (attribute.specified) {
                // Issue 468: Chrome "Click to Active Flash" box stomps on title attribute
                if (
                    attribute.name === "title" &&
                    attribute.value === "Adobe Flash Player"
                ) {
                    continue;
                }

                try {
                    destination.setAttribute(attribute.name, attribute.value);
                } catch (err) {
                    // The embed may have invalid attributes, so handle these gracefully.
                    console.warn(
                        `Unable to set attribute ${attribute.name} on Ruffle instance`,
                    );
                }
            }
        }

        for (const node of Array.from(element.children)) {
            destination.appendChild(node);
        }
    }
}
