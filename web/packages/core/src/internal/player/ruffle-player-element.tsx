import type { DataLoadOptions, URLLoadOptions } from "../../load-options";
import type { MovieMetadata } from "../../movie-metadata";
import { InnerPlayer, ReadyState } from "./inner";
import { Player } from "../../public/player";

/**
 * The ruffle player element that should be inserted onto the page.
 *
 * This element will represent the rendered and intractable flash movie.
 */
export class RufflePlayerElement extends HTMLElement implements Player {
    #inner: InnerPlayer;

    get onFSCommand(): ((command: string, args: string) => boolean) | null {
        return this.#inner.onFSCommand;
    }

    set onFSCommand(
        value: ((command: string, args: string) => boolean) | null,
    ) {
        this.#inner.onFSCommand = value;
    }

    get readyState(): ReadyState {
        return this.#inner._readyState;
    }

    get metadata(): MovieMetadata | null {
        return this.#inner.metadata;
    }

    constructor() {
        super();
        this.#inner = new InnerPlayer(
            this,
            () => this.debugPlayerInfo(),
            (name) => {
                try {
                    Object.defineProperty(this, name, {
                        value: (...args: unknown[]) => {
                            return this.#inner.callExternalInterface(
                                name,
                                args,
                            );
                        },
                        configurable: true,
                    });
                } catch (error) {
                    console.warn(
                        `Error setting ExternalInterface legacy callback for ${name}`,
                        error,
                    );
                }
            },
        );
    }

    get loadedConfig(): URLLoadOptions | DataLoadOptions | null {
        return this.#inner.loadedConfig ?? null;
    }

    connectedCallback(): void {
        this.#inner.updateStyles();
    }

    static get observedAttributes(): string[] {
        return ["width", "height"];
    }

    attributeChangedCallback(
        name: string,
        _oldValue: string | undefined,
        _newValue: string | undefined,
    ): void {
        if (name === "width" || name === "height") {
            this.#inner.updateStyles();
        }
    }

    disconnectedCallback(): void {
        this.#inner.destroy();
    }

    async reload(): Promise<void> {
        await this.#inner.reload();
    }

    async load(
        options: string | URLLoadOptions | DataLoadOptions,
        isPolyfillElement: boolean = false,
    ): Promise<void> {
        await this.#inner.load(options, isPolyfillElement);
    }

    play(): void {
        this.#inner.play();
    }

    get isPlaying(): boolean {
        return this.#inner.isPlaying;
    }

    get volume(): number {
        return this.#inner.volume;
    }

    set volume(value: number) {
        this.#inner.volume = value;
    }

    get fullscreenEnabled(): boolean {
        return this.#inner.fullscreenEnabled;
    }

    get isFullscreen(): boolean {
        return this.#inner.isFullscreen;
    }

    setFullscreen(isFull: boolean): void {
        this.#inner.setFullscreen(isFull);
    }

    enterFullscreen(): void {
        this.#inner.enterFullscreen();
    }

    exitFullscreen(): void {
        this.#inner.exitFullscreen();
    }

    async downloadSwf(): Promise<void> {
        await this.#inner.downloadSwf();
    }

    pause(): void {
        this.#inner.pause();
    }

    set traceObserver(observer: ((message: string) => void) | null) {
        this.#inner.traceObserver = observer;
    }

    protected debugPlayerInfo(): string {
        return "";
    }

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
