import type { RuffleHandle, ZipWriter } from "../../../dist/ruffle_web";
import {
    AutoPlay,
    ContextMenu,
    DataLoadOptions,
    NetworkingAccessMode,
    UnmuteOverlay,
    URLLoadOptions,
    WindowMode,
    DEFAULT_CONFIG,
} from "../../public/config";
import type { MovieMetadata } from "../../public/player";
import { ruffleShadowTemplate } from "../ui/shadow-template";
import { text, textAsParagraphs } from "../i18n";
import { swfFileName } from "../../swf-utils";
import { isExtension } from "../../current-script";
import { buildInfo } from "../../build-info";
import { RUFFLE_ORIGIN } from "../constants";
import {
    InvalidOptionsError,
    InvalidSwfError,
    LoadRuffleWasmError,
    LoadSwfError,
} from "../errors";
import { showPanicScreen } from "../ui/panic";
import { createRuffleBuilder } from "../../load-ruffle";
import { lookupElement } from "../register-element";
import { configureBuilder } from "../builder";

const DIMENSION_REGEX = /^\s*(\d+(\.\d+)?(%)?)/;

let isAudioContextUnmuted = false;

// Safari still requires prefixed fullscreen APIs, see:
// https://developer.mozilla.org/en-US/docs/Web/API/Element/requestFullScreen
// Safari uses alternate capitalization of FullScreen in some older APIs.
declare global {
    interface Document {
        webkitFullscreenEnabled?: boolean;
        webkitFullscreenElement?: boolean;
        webkitExitFullscreen?: () => void;
        webkitCancelFullScreen?: () => void;
    }
    interface Element {
        /**
         * @ignore
         */
        webkitRequestFullscreen?: (options: unknown) => unknown;
        /**
         * @ignore
         */
        webkitRequestFullScreen?: (options: unknown) => unknown;
    }
}

/**
 * An item to show in Ruffle's custom context menu.
 */
interface ContextMenuItem {
    /**
     * The text shown to the user.
     */
    text: string;

    /**
     * The function to call when clicked.
     *
     * @param event The mouse event that triggered the click.
     */
    onClick: (event: MouseEvent) => Promise<void>;

    /**
     * Whether this item is clickable.
     *
     * @default true
     */
    enabled?: boolean;
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
        | null,
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

class Point {
    constructor(
        public x: number,
        public y: number,
    ) {}

    distanceTo(other: Point) {
        const dx = other.x - this.x;
        const dy = other.y - this.y;
        return Math.sqrt(dx * dx + dy * dy);
    }
}

/**
 * This is the backing logic behind a HTML "player" element, and bridges the gap to the Rust codebase.
 */
export class InnerPlayer {
    /**
     * Triggered when a movie metadata has been loaded (such as movie width and height).
     *
     * @event RufflePlayer#loadedmetadata
     */
    static LOADED_METADATA = "loadedmetadata";

    /**
     * Triggered when a movie is fully loaded.
     *
     * @event RufflePlayer#loadeddata
     */
    static LOADED_DATA = "loadeddata";

    // The element that contains this player
    public element: HTMLElement;

    private readonly shadow: ShadowRoot;
    private readonly dynamicStyles: HTMLStyleElement;
    private readonly container: HTMLElement;
    private readonly playButton: HTMLElement;
    private readonly unmuteOverlay: HTMLElement;
    private readonly splashScreen: HTMLElement;
    private readonly virtualKeyboard: HTMLInputElement;
    private readonly saveManager: HTMLDivElement;
    private readonly volumeControls: HTMLDivElement;
    private readonly videoModal: HTMLDivElement;
    private readonly hardwareAccelerationModal: HTMLDivElement;
    private readonly clipboardModal: HTMLDivElement;

    private readonly contextMenuOverlay: HTMLElement;
    // Firefox has a read-only "contextMenu" property,
    // so avoid shadowing it.
    private readonly contextMenuElement: HTMLElement;

    // Allows the user to permanently disable the context menu.
    private contextMenuForceDisabled = false;

    // Whether the most recent pointer event was from a touch (or pen).
    private isTouch = false;
    // Whether this device sends contextmenu events.
    // Set to true when a contextmenu event is seen.
    private contextMenuSupported = false;

    // When set to `true`, the next context menu event will
    // not show the context menu.
    private _suppressContextMenu = false;

    // The effective config loaded upon `.load()`.
    public loadedConfig?: URLLoadOptions | DataLoadOptions;

    private swfUrl?: URL;
    private instance: RuffleHandle | null;
    private newZipWriter: (() => ZipWriter) | null;
    private lastActivePlayingState: boolean;

    metadata: MovieMetadata | null;
    _readyState: ReadyState;

    private panicked = false;
    private rendererDebugInfo = "";

    private longPressTimer: ReturnType<typeof setTimeout> | null = null;
    private pointerDownPosition: Point | null = null;
    private pointerMoveMaxDistance = 0;

    private volumeSettings: VolumeControls;
    private readonly debugPlayerInfo: () => string;
    protected readonly onCallbackAvailable: (name: string) => void;

    public constructor(
        element: HTMLElement,
        debugPlayerInfo: () => string,
        onCallbackAvailable: (name: string) => void,
    ) {
        this.element = element;
        this.debugPlayerInfo = debugPlayerInfo;
        this.onCallbackAvailable = onCallbackAvailable;

        this.shadow = this.element.attachShadow({ mode: "open", delegatesFocus: true });
        this.shadow.appendChild(ruffleShadowTemplate.content.cloneNode(true));

        this.dynamicStyles = this.shadow.getElementById(
            "dynamic-styles",
        ) as HTMLStyleElement;
        this.container = this.shadow.getElementById("container")!;
        this.playButton = this.shadow.getElementById("play-button")!;
        this.playButton.addEventListener("click", () => this.play());

        this.unmuteOverlay = this.shadow.getElementById("unmute-overlay")!;
        this.splashScreen = this.shadow.getElementById("splash-screen")!;
        this.virtualKeyboard = this.shadow.getElementById(
            "virtual-keyboard",
        )! as HTMLInputElement;
        this.virtualKeyboard.addEventListener(
            "input",
            this.virtualKeyboardInput.bind(this),
        );
        this.saveManager = this.shadow.getElementById(
            "save-manager",
        )! as HTMLDivElement;
        this.videoModal = this.shadow.getElementById(
            "video-modal",
        )! as HTMLDivElement;
        this.hardwareAccelerationModal = this.shadow.getElementById(
            "hardware-acceleration-modal",
        )! as HTMLDivElement;
        this.volumeControls = this.shadow.getElementById(
            "volume-controls-modal",
        ) as HTMLDivElement;
        this.clipboardModal = this.shadow.getElementById(
            "clipboard-modal",
        ) as HTMLDivElement;
        this.addModalJavaScript(this.saveManager);
        this.addModalJavaScript(this.volumeControls);
        this.addModalJavaScript(this.videoModal);
        this.addModalJavaScript(this.hardwareAccelerationModal);
        this.addModalJavaScript(this.clipboardModal);

        this.volumeSettings = new VolumeControls(false, 100);
        this.addVolumeControlsJavaScript(this.volumeControls);

        const backupSaves = this.saveManager.querySelector(
            ".modal-button",
        ) as HTMLElement;
        if (backupSaves) {
            backupSaves.addEventListener("click", this.backupSaves.bind(this));
            backupSaves.innerText = text("save-backup-all");
        }

        const unmuteSvg = this.unmuteOverlay.querySelector(
            "#unmute-overlay-svg",
        ) as SVGElement;
        if (unmuteSvg) {
            const unmuteText = unmuteSvg.querySelector(
                "#unmute-text",
            ) as SVGTextElement;
            unmuteText.textContent = text("click-to-unmute");
        }

        this.contextMenuOverlay = this.shadow.getElementById(
            "context-menu-overlay",
        )!;
        this.contextMenuElement = this.shadow.getElementById("context-menu")!;
        const preserveMenu = (event: MouseEvent) => {
            event.preventDefault();
            event.stopPropagation();
        };
        this.contextMenuElement.addEventListener("contextmenu", preserveMenu);
        this.contextMenuElement.addEventListener("click", preserveMenu);

        document.documentElement.addEventListener(
            "pointerdown",
            this.checkIfTouch.bind(this),
        );
        this.element.addEventListener(
            "contextmenu",
            this.showContextMenu.bind(this),
        );
        this.container.addEventListener(
            "pointerdown",
            this.pointerDown.bind(this),
        );
        this.container.addEventListener(
            "pointermove",
            this.checkLongPressMovement.bind(this),
        );
        this.container.addEventListener(
            "pointerup",
            this.checkLongPress.bind(this),
        );
        this.container.addEventListener(
            "pointercancel",
            this.clearLongPressTimer.bind(this),
        );

        this.element.addEventListener(
            "fullscreenchange",
            this.fullScreenChange.bind(this),
        );
        this.element.addEventListener(
            "webkitfullscreenchange",
            this.fullScreenChange.bind(this),
        );

        this.instance = null;
        this.newZipWriter = null;
        this.onFSCommand = null;

        this._readyState = ReadyState.HaveNothing;
        this.metadata = null;

        this.lastActivePlayingState = false;
        this.setupPauseOnTabHidden();
    }

    /**
     * A movie can communicate with the hosting page using fscommand
     * as long as script access is allowed.
     *
     * @param command A string passed to the host application for any use.
     * @param args A string passed to the host application for any use.
     * @returns True if the command was handled.
     */
    onFSCommand: ((command: string, args: string) => boolean) | null;

    /**
     * Any configuration that should apply to this specific player.
     * This will be defaulted with any global configuration.
     */
    config: URLLoadOptions | DataLoadOptions | object = {};

    /**
     * Add functions to open and close a modal.
     *
     * @param modalElement The element containing the modal.
     */
    private addModalJavaScript(modalElement: HTMLDivElement): void {
        const videoHolder = modalElement.querySelector("#video-holder");
        const hideModal = () => {
            modalElement.classList.add("hidden");
            if (videoHolder) {
                videoHolder.textContent = "";
            }
        };

        modalElement.parentNode!.addEventListener("click", hideModal);
        const modalArea = modalElement.querySelector(".modal-area");
        if (modalArea) {
            modalArea.addEventListener("click", (event) =>
                event.stopPropagation(),
            );
        }
        const closeModal = modalElement.querySelector(".close-modal");
        if (closeModal) {
            closeModal.addEventListener("click", hideModal);
        }
    }

    /**
     * Add the volume control texts, set the controls to the current settings and
     * add event listeners to update the settings and controls when being changed.
     *
     * @param volumeControlsModal The element containing the volume controls modal.
     */
    private addVolumeControlsJavaScript(
        volumeControlsModal: HTMLDivElement,
    ): void {
        const volumeMuteCheckbox = volumeControlsModal.querySelector(
            "#mute-checkbox",
        ) as HTMLInputElement;
        const volumeMuteIcon = volumeControlsModal.querySelector(
            "#volume-mute",
        ) as HTMLLabelElement;
        const volumeIcons = [
            volumeControlsModal.querySelector(
                "#volume-min",
            ) as HTMLLabelElement,
            volumeControlsModal.querySelector(
                "#volume-mid",
            ) as HTMLLabelElement,
            volumeControlsModal.querySelector(
                "#volume-max",
            ) as HTMLLabelElement,
        ];
        const volumeSlider = volumeControlsModal.querySelector(
            "#volume-slider",
        ) as HTMLInputElement;
        const volumeSliderText = volumeControlsModal.querySelector(
            "#volume-slider-text",
        ) as HTMLSpanElement;

        const setVolumeIcon = () => {
            if (this.volumeSettings.isMuted) {
                volumeMuteIcon.style.display = "inline";
                volumeIcons.forEach((icon) => {
                    icon.style.display = "none";
                });
            } else {
                volumeMuteIcon.style.display = "none";
                const iconIndex = Math.round(this.volumeSettings.volume / 50);
                volumeIcons.forEach((icon, i) => {
                    icon.style.display = i === iconIndex ? "inline" : "none";
                });
            }
        };

        // Set the controls to the current settings.
        volumeMuteCheckbox.checked = this.volumeSettings.isMuted;
        volumeSlider.disabled = volumeMuteCheckbox.checked;
        volumeSlider.valueAsNumber = this.volumeSettings.volume;
        volumeSliderText.textContent = volumeSlider.value + "%";
        setVolumeIcon();

        // Add event listeners to update the settings and controls.
        volumeMuteCheckbox.addEventListener("change", () => {
            volumeSlider.disabled = volumeMuteCheckbox.checked;
            this.volumeSettings.isMuted = volumeMuteCheckbox.checked;
            this.instance?.set_volume(this.volumeSettings.get_volume());
            setVolumeIcon();
        });
        volumeSlider.addEventListener("input", () => {
            volumeSliderText.textContent = volumeSlider.value + "%";
            this.volumeSettings.volume = volumeSlider.valueAsNumber;
            this.instance?.set_volume(this.volumeSettings.get_volume());
            setVolumeIcon();
        });
    }

    /**
     * Setup event listener to detect when tab is not active to pause instance playback.
     * this.instance.play() is called when the tab becomes visible only if the
     * the instance was not paused before tab became hidden.
     *
     * See: https://developer.mozilla.org/en-US/docs/Web/API/Page_Visibility_API
     * @ignore
     * @internal
     */
    private setupPauseOnTabHidden(): void {
        document.addEventListener(
            "visibilitychange",
            () => {
                if (!this.instance) {
                    return;
                }

                // Tab just changed to be inactive. Record whether instance was playing.
                if (document.hidden) {
                    this.lastActivePlayingState = this.instance.is_playing();
                    this.instance.pause();
                }
                // Play only if instance was playing originally.
                if (!document.hidden && this.lastActivePlayingState === true) {
                    this.instance.play();
                }
            },
            false,
        );
    }

    /**
     * Updates the internal shadow DOM to reflect any set attributes from
     * this element.
     */
    updateStyles(): void {
        if (this.dynamicStyles.sheet) {
            if (this.dynamicStyles.sheet.cssRules) {
                for (
                    let i = this.dynamicStyles.sheet.cssRules.length - 1;
                    i >= 0;
                    i--
                ) {
                    this.dynamicStyles.sheet.deleteRule(i);
                }
            }

            const widthAttr = this.element.attributes.getNamedItem("width");
            if (widthAttr !== undefined && widthAttr !== null) {
                const width = InnerPlayer.htmlDimensionToCssDimension(
                    widthAttr.value,
                );
                if (width !== null) {
                    this.dynamicStyles.sheet.insertRule(
                        `:host { width: ${width}; }`,
                    );
                }
            }

            const heightAttr = this.element.attributes.getNamedItem("height");
            if (heightAttr !== undefined && heightAttr !== null) {
                const height = InnerPlayer.htmlDimensionToCssDimension(
                    heightAttr.value,
                );
                if (height !== null) {
                    this.dynamicStyles.sheet.insertRule(
                        `:host { height: ${height}; }`,
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
        const element = lookupElement("ruffle-object");

        if (element !== null) {
            let parent = this.element.parentNode;
            while (parent !== document && parent !== null) {
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
        this.destroy();

        if (
            this.loadedConfig &&
            this.loadedConfig.splashScreen !== false &&
            this.loadedConfig.preloader !== false
        ) {
            this.showSplashScreen();
        }
        if (this.loadedConfig && this.loadedConfig.preloader === false) {
            console.warn(
                "The configuration option preloader has been replaced with splashScreen. If you own this website, please update the configuration.",
            );
        }
        if (
            this.loadedConfig &&
            this.loadedConfig.maxExecutionDuration &&
            typeof this.loadedConfig.maxExecutionDuration !== "number"
        ) {
            console.warn(
                "Configuration: An obsolete format for duration for 'maxExecutionDuration' was used, " +
                    "please use a single number indicating seconds instead. For instance '15' instead of " +
                    "'{secs: 15, nanos: 0}'.",
            );
        }
        if (
            this.loadedConfig &&
            typeof this.loadedConfig.contextMenu === "boolean"
        ) {
            console.warn(
                'The configuration option contextMenu no longer takes a boolean. Use "on", "off", or "rightClickOnly".',
            );
        }

        const [builder, zipWriterClass] = await createRuffleBuilder(
            this.onRuffleDownloadProgress.bind(this),
        ).catch((e) => {
            console.error(`Serious error loading Ruffle: ${e}`);
            const error = new LoadRuffleWasmError(e);
            this.panic(error);
            throw error;
        });
        this.newZipWriter = zipWriterClass;
        configureBuilder(builder, this.loadedConfig || {});
        builder.setVolume(this.volumeSettings.get_volume());

        if (this.loadedConfig?.fontSources) {
            for (const url of this.loadedConfig.fontSources) {
                try {
                    const response = await fetch(url);
                    builder.addFont(
                        url,
                        new Uint8Array(await response.arrayBuffer()),
                    );
                } catch (error) {
                    console.warn(
                        `Couldn't download font source from ${url}`,
                        error,
                    );
                }
            }
        }

        for (const key in this.loadedConfig?.defaultFonts) {
            const names = (
                this.loadedConfig.defaultFonts as {
                    [key: string]: Array<string>;
                }
            )[key];
            if (names) {
                builder.setDefaultFont(key, names);
            }
        }

        this.instance = await builder.build(this.container, this).catch((e) => {
            console.error(`Serious error loading Ruffle: ${e}`);
            this.panic(e);
            throw e;
        });

        this.rendererDebugInfo = this.instance!.renderer_debug_info();

        if (this.rendererDebugInfo.includes("Adapter Device Type: Cpu")) {
            this.container.addEventListener(
                "mouseover",
                this.openHardwareAccelerationModal.bind(this),
                {
                    once: true,
                },
            );
        }

        const actuallyUsedRendererName = this.instance!.renderer_name();
        const constructor = this.instance!.constructor as typeof RuffleHandle;

        console.log(
            "%c" +
                "New Ruffle instance created (Version: " +
                buildInfo.versionName +
                " | WebAssembly extensions: " +
                (constructor.is_wasm_simd_used() ? "ON" : "OFF") +
                " | Used renderer: " +
                (actuallyUsedRendererName ?? "") +
                ")",
            "background: #37528C; color: #FFAD33",
        );

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

        this.unmuteAudioContext();

        // Treat invalid values as `AutoPlay.Auto`.
        if (
            !this.loadedConfig ||
            this.loadedConfig.autoplay === AutoPlay.On ||
            (this.loadedConfig.autoplay !== AutoPlay.Off &&
                this.audioState() === "running")
        ) {
            this.play();

            if (this.audioState() !== "running") {
                // Treat invalid values as `UnmuteOverlay.Visible`.
                if (
                    !this.loadedConfig ||
                    this.loadedConfig.unmuteOverlay !== UnmuteOverlay.Hidden
                ) {
                    this.unmuteOverlay.style.display = "block";
                }

                this.container.addEventListener(
                    "click",
                    this.unmuteOverlayClicked.bind(this),
                    {
                        once: true,
                    },
                );

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
     * Uploads the splash screen progress bar.
     *
     * @param bytesLoaded The size of the Ruffle WebAssembly file downloaded so far.
     * @param bytesTotal The total size of the Ruffle WebAssembly file.
     */
    private onRuffleDownloadProgress(bytesLoaded: number, bytesTotal: number) {
        const loadBar = this.splashScreen.querySelector(
            ".loadbar-inner",
        ) as HTMLElement;
        const outerLoadbar = this.splashScreen.querySelector(
            ".loadbar",
        ) as HTMLElement;
        if (Number.isNaN(bytesTotal)) {
            if (outerLoadbar) {
                outerLoadbar.style.display = "none";
            }
        } else {
            loadBar.style.width = `${100.0 * (bytesLoaded / bytesTotal)}%`;
        }
    }

    /**
     * Destroys the currently running instance of Ruffle.
     */
    destroy(): void {
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
            this.metadata = null;
            this._readyState = ReadyState.HaveNothing;
            console.log("Ruffle instance destroyed.");
        }
    }

    private checkOptions(
        options: string | URLLoadOptions | DataLoadOptions,
    ): URLLoadOptions | DataLoadOptions {
        if (typeof options === "string") {
            return { url: options };
        }

        const check: (
            condition: boolean,
            message: string,
        ) => asserts condition = (condition, message) => {
            if (!condition) {
                const error = new InvalidOptionsError(message);
                this.panic(error);
                throw error;
            }
        };
        check(
            options !== null && typeof options === "object",
            "Argument 0 must be a string or object",
        );
        check(
            "url" in options || "data" in options,
            "Argument 0 must contain a `url` or `data` key",
        );
        check(
            !("url" in options) || typeof options.url === "string",
            "`url` must be a string",
        );
        return options;
    }

    /**
     * Reloads the player, as if you called {@link RufflePlayer.load} with the same config as the last time it was called.
     *
     * If this player has never been loaded, this method will return an error.
     */
    async reload(): Promise<void> {
        if (this.loadedConfig) {
            await this.load(this.loadedConfig);
        } else {
            throw new Error("Cannot reload if load wasn't first called");
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
        options = this.checkOptions(options);

        if (!this.element.isConnected || this.isUnusedFallbackObject()) {
            console.warn(
                "Ignoring attempt to play a disconnected or suspended Ruffle element",
            );
            return;
        }

        if (isFallbackElement(this.element)) {
            // Silently fail on attempt to play a Ruffle element inside a specific node.
            return;
        }

        try {
            this.loadedConfig = {
                ...DEFAULT_CONFIG,
                // The default allowScriptAccess value for polyfilled elements is samedomain.
                ...(isPolyfillElement && "url" in options
                    ? {
                          allowScriptAccess: parseAllowScriptAccess(
                              "samedomain",
                              options.url,
                          )!,
                      }
                    : {}),
                ...(window.RufflePlayer?.config ?? {}),
                ...this.config,
                ...options,
            };

            // Pre-emptively set background color of container while Ruffle/SWF loads.
            if (
                this.loadedConfig.backgroundColor &&
                this.loadedConfig.wmode !== WindowMode.Transparent
            ) {
                this.container.style.backgroundColor =
                    this.loadedConfig.backgroundColor;
            }

            await this.ensureFreshInstance();

            if ("url" in options) {
                console.log(`Loading SWF file ${options.url}`);
                this.swfUrl = new URL(options.url, document.baseURI);

                this.instance!.stream_from(
                    this.swfUrl.href,
                    sanitizeParameters(options.parameters),
                );
            } else if ("data" in options) {
                console.log("Loading SWF data");
                delete this.swfUrl;
                this.instance!.load_data(
                    new Uint8Array(options.data),
                    sanitizeParameters(options.parameters),
                    options.swfFileName || "movie.swf",
                );
            }
        } catch (e) {
            console.error(`Serious error occurred loading SWF file: ${e}`);
            const err = new Error(e as string);
            this.panic(err);
            throw err;
        }
    }

    /**
     * Plays or resumes the movie.
     */
    play(): void {
        if (this.instance) {
            this.instance.play();
            this.playButton.style.display = "none";
        }
    }

    /**
     * Whether this player is currently playing.
     *
     * @returns True if this player is playing, false if it's paused or hasn't started yet.
     */
    get isPlaying(): boolean {
        if (this.instance) {
            return this.instance.is_playing();
        }
        return false;
    }

    /**
     * Returns the master volume of the player.
     *
     * The volume is linear and not adapted for logarithmic hearing.
     *
     * @returns The volume. 1.0 is 100% volume.
     */
    get volume(): number {
        if (this.instance) {
            return this.instance.volume();
        }
        return 1.0;
    }

    /**
     * Sets the master volume of the player.
     *
     * The volume should be linear and not adapted for logarithmic hearing.
     *
     * @param value The volume. 1.0 is 100% volume.
     */
    set volume(value: number) {
        if (this.instance) {
            this.instance.set_volume(value);
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
            this.element
        );
    }

    /**
     * Exported function that requests the browser to change the fullscreen state if
     * it is allowed.
     *
     * @param isFull Whether to set to fullscreen or return to normal.
     */
    setFullscreen(isFull: boolean): void {
        if (this.fullscreenEnabled && isFull !== this.isFullscreen) {
            if (isFull) {
                this.enterFullscreen();
            } else {
                this.exitFullscreen();
            }
        }
    }

    /**
     * Requests the browser to make this player fullscreen.
     *
     * This is not guaranteed to succeed, please check [[fullscreenEnabled]] first.
     */
    enterFullscreen(): void {
        const options: FullscreenOptions = {
            navigationUI: "hide",
        };
        if (this.element.requestFullscreen) {
            this.element.requestFullscreen(options);
        } else if (this.element.webkitRequestFullscreen) {
            this.element.webkitRequestFullscreen(options);
        } else if (this.element.webkitRequestFullScreen) {
            this.element.webkitRequestFullScreen(options);
        }
    }

    /**
     * Requests the browser to no longer make this player fullscreen.
     */
    exitFullscreen(): void {
        if (document.exitFullscreen) {
            document.exitFullscreen();
        } else if (document.webkitExitFullscreen) {
            document.webkitExitFullscreen();
        } else if (document.webkitCancelFullScreen) {
            document.webkitCancelFullScreen();
        }
    }

    /**
     * Called when entering / leaving fullscreen.
     */
    private fullScreenChange(): void {
        this.instance?.set_fullscreen(this.isFullscreen);
    }

    /**
     * Prompt the user to download a file.
     *
     * @param blob The content to download.
     * @param name The name to give the file.
     */
    private saveFile(blob: Blob, name: string): void {
        const blobURL = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = blobURL;
        link.download = name;
        link.click();
        URL.revokeObjectURL(blobURL);
    }

    private checkIfTouch(event: PointerEvent): void {
        this.isTouch =
            event.pointerType === "touch" || event.pointerType === "pen";
    }

    private base64ToArray(bytesBase64: string): Uint8Array {
        const byteString = atob(bytesBase64);
        const ia = new Uint8Array(byteString.length);
        for (let i = 0; i < byteString.length; i++) {
            ia[i] = byteString.charCodeAt(i);
        }
        return ia;
    }

    private base64ToBlob(bytesBase64: string, mimeString: string): Blob {
        const ab = this.base64ToArray(bytesBase64);
        const blob = new Blob([ab], { type: mimeString });
        return blob;
    }

    /**
     * @returns If the string represent a base-64 encoded SOL file
     * Check if string is a base-64 encoded SOL file
     * @param solData The base-64 encoded SOL string
     */
    private isB64SOL(solData: string): boolean {
        try {
            const decodedData = atob(solData);
            return decodedData.slice(6, 10) === "TCSO";
        } catch (e) {
            return false;
        }
    }

    private confirmReloadSave(
        solKey: string,
        b64SolData: string,
        replace: boolean,
    ) {
        if (this.isB64SOL(b64SolData)) {
            if (localStorage[solKey]) {
                if (!replace) {
                    const confirmDelete = confirm(text("save-delete-prompt"));
                    if (!confirmDelete) {
                        return;
                    }
                }
                const swfPath = this.swfUrl ? this.swfUrl.pathname : "";
                const swfHost = this.swfUrl
                    ? this.swfUrl.hostname
                    : document.location.hostname;
                const savePath = solKey.split("/").slice(1, -1).join("/");
                if (swfPath.includes(savePath) && solKey.startsWith(swfHost)) {
                    const confirmReload = confirm(
                        text("save-reload-prompt", {
                            action: replace ? "replace" : "delete",
                        }),
                    );
                    if (confirmReload && this.loadedConfig) {
                        this.destroy();
                        replace
                            ? localStorage.setItem(solKey, b64SolData)
                            : localStorage.removeItem(solKey);
                        this.reload();
                        this.populateSaves();
                        this.saveManager.classList.add("hidden");
                    }
                    return;
                }
                replace
                    ? localStorage.setItem(solKey, b64SolData)
                    : localStorage.removeItem(solKey);
                this.populateSaves();
                this.saveManager.classList.add("hidden");
            }
        }
    }

    /**
     * Replace save from SOL file.
     *
     * @param event The change event fired
     * @param solKey The localStorage save file key
     */
    private replaceSOL(event: Event, solKey: string): void {
        const fileInput = event.target as HTMLInputElement;
        const reader = new FileReader();
        reader.addEventListener("load", () => {
            if (reader.result && typeof reader.result === "string") {
                const b64Regex = new RegExp("data:.*;base64,");
                const b64SolData = reader.result.replace(b64Regex, "");
                this.confirmReloadSave(solKey, b64SolData, true);
            }
        });
        if (
            fileInput &&
            fileInput.files &&
            fileInput.files.length > 0 &&
            fileInput.files[0]
        ) {
            reader.readAsDataURL(fileInput.files[0]);
        }
    }

    /**
     * Check if there are any saves.
     *
     * @returns True if there is at least one save.
     */
    private checkSaves(): boolean {
        if (!this.saveManager.querySelector("#local-saves")) {
            return false;
        }
        try {
            if (localStorage === null) {
                return false;
            }
        } catch (e: unknown) {
            return false;
        }
        return Object.keys(localStorage).some((key) => {
            const solName = key.split("/").pop();
            const solData = localStorage.getItem(key);
            return solName && solData && this.isB64SOL(solData);
        });
    }

    /**
     * Delete local save.
     *
     * @param key The key to remove from local storage
     */
    private deleteSave(key: string): void {
        const b64SolData = localStorage.getItem(key);
        if (b64SolData) {
            this.confirmReloadSave(key, b64SolData, false);
        }
    }

    /**
     * Puts the local save SOL file keys in a table.
     */
    private populateSaves(): void {
        if (!this.checkSaves()) {
            return;
        }
        const saveTable = this.saveManager.querySelector("#local-saves")!;
        saveTable.textContent = "";
        Object.keys(localStorage).forEach((key) => {
            const solName = key.split("/").pop();
            const solData = localStorage.getItem(key);
            if (solName && solData && this.isB64SOL(solData)) {
                const row = document.createElement("TR");
                const keyCol = document.createElement("TD");
                keyCol.textContent = solName;
                keyCol.title = key;
                const downloadCol = document.createElement("TD");
                const downloadSpan = document.createElement("SPAN");
                downloadSpan.className = "save-option";
                downloadSpan.id = "download-save";
                downloadSpan.title = text("save-download");
                downloadSpan.addEventListener("click", () => {
                    const blob = this.base64ToBlob(
                        solData,
                        "application/octet-stream",
                    );
                    this.saveFile(blob, solName + ".sol");
                });
                downloadCol.appendChild(downloadSpan);
                const replaceCol = document.createElement("TD");
                const replaceInput = document.createElement(
                    "INPUT",
                ) as HTMLInputElement;
                replaceInput.type = "file";
                replaceInput.accept = ".sol";
                replaceInput.className = "replace-save";
                replaceInput.id = "replace-save-" + key;
                const replaceLabel = document.createElement(
                    "LABEL",
                ) as HTMLLabelElement;
                replaceLabel.htmlFor = "replace-save-" + key;
                replaceLabel.className = "save-option";
                replaceLabel.id = "replace-save";
                replaceLabel.title = text("save-replace");
                replaceInput.addEventListener("change", (event) =>
                    this.replaceSOL(event, key),
                );
                replaceCol.appendChild(replaceInput);
                replaceCol.appendChild(replaceLabel);
                const deleteCol = document.createElement("TD");
                const deleteSpan = document.createElement("SPAN");
                deleteSpan.className = "save-option";
                deleteSpan.id = "delete-save";
                deleteSpan.title = text("save-delete");
                deleteSpan.addEventListener("click", () =>
                    this.deleteSave(key),
                );
                deleteCol.appendChild(deleteSpan);
                row.appendChild(keyCol);
                row.appendChild(downloadCol);
                row.appendChild(replaceCol);
                row.appendChild(deleteCol);
                saveTable.appendChild(row);
            }
        });
    }

    /**
     * Gets the local save information as SOL files and downloads them as a single ZIP file.
     */
    private async backupSaves(): Promise<void> {
        const zip = this.newZipWriter!();
        const duplicateNames: string[] = [];
        Object.keys(localStorage).forEach((key) => {
            let solName = String(key.split("/").pop());
            const solData = localStorage.getItem(key);
            if (solData && this.isB64SOL(solData)) {
                const array = this.base64ToArray(solData);
                const duplicate = duplicateNames.filter(
                    (value) => value === solName,
                ).length;
                duplicateNames.push(solName);
                if (duplicate > 0) {
                    solName += ` (${duplicate + 1})`;
                }
                zip.addFile(solName + ".sol", array);
            }
        });
        const blob = new Blob([zip.save()], { type: "application/zip" });
        this.saveFile(blob, "saves.zip");
    }

    /**
     * Opens the hardware acceleration info modal.
     */
    private openHardwareAccelerationModal(): void {
        this.hardwareAccelerationModal.classList.remove("hidden");
    }

    /**
     * Opens the save manager.
     */
    private async openSaveManager(): Promise<void> {
        this.populateSaves();
        this.saveManager.classList.remove("hidden");
    }

    /**
     * Opens the volume controls.
     */
    private openVolumeControls(): void {
        this.volumeControls.classList.remove("hidden");
    }

    /**
     * Fetches the loaded SWF and downloads it.
     */
    async downloadSwf(): Promise<void> {
        try {
            if (this.swfUrl) {
                console.log("Downloading SWF: " + this.swfUrl);
                const response = await fetch(this.swfUrl.href);
                if (!response.ok) {
                    console.error("SWF download failed");
                    return;
                }
                const blob = await response.blob();
                this.saveFile(blob, swfFileName(this.swfUrl));
            } else {
                console.error("SWF download failed");
            }
        } catch (err) {
            console.error("SWF download failed");
        }
    }

    private virtualKeyboardInput() {
        const input = this.virtualKeyboard;
        const string = input.value;
        for (const char of string) {
            for (const eventType of ["keydown", "keyup"]) {
                this.element.dispatchEvent(
                    new KeyboardEvent(eventType, {
                        key: char,
                        bubbles: true,
                    }),
                );
            }
        }
        input.value = "";
    }

    protected openVirtualKeyboard(): void {
        // Virtual keyboard is opened/closed synchronously from core,
        // and opening/closing it is basically dispatching
        // focus events (which may also be dispatched to the player).
        // In order not to deadlock here (or rather throw an error),
        // these actions should be performed asynchronously.
        // However, some browsers (i.e. Safari) require user interaction
        // in order to open the virtual keyboard.
        // That is why we are checking whether Ruffle already has focus:
        //  1. if it does, no focus events will be dispatched to
        //     the player when we focus the virtual keyboard, and
        //  2. if it doesn't, the action shouldn't be a result of user
        //     interaction and focusing synchronously wouldn't work anyway.
        if (this.instance?.has_focus()) {
            this.virtualKeyboard.focus({ preventScroll: true });
        } else {
            setTimeout(() => {
                this.virtualKeyboard.focus({ preventScroll: true });
            }, 0);
        }
    }

    protected closeVirtualKeyboard(): void {
        // Note that closing the keyboard is a little tricky, as we cannot
        // just remove the focus here, as the player should still be focused.
        // We want to switch the focus to the container instead, but the user may also
        // click away from the player, and in that case we do not want to re-focus it.
        // We also have to take into account that the keyboard may be
        // closed even if the player doesn't have focus at all.
        // That's why we have to "transfer" the focus from the keyboard to the container.
        if (this.isVirtualKeyboardFocused()) {
            this.container.focus({ preventScroll: true });
        }
    }

    protected isVirtualKeyboardFocused(): boolean {
        return this.shadow.activeElement === this.virtualKeyboard;
    }

    private contextMenuItems(): Array<ContextMenuItem | null> {
        const CHECKMARK = String.fromCharCode(0x2713);
        const items: Array<ContextMenuItem | null> = [];
        const addSeparator = () => {
            // Don't start with or duplicate separators.
            if (items.length > 0 && items[items.length - 1] !== null) {
                items.push(null);
            }
        };

        if (this.instance && this.isPlaying) {
            const customItems: {
                readonly caption: string;
                readonly checked: boolean;
                readonly enabled: boolean;
                readonly separatorBefore: boolean;
            }[] = this.instance.prepare_context_menu();
            customItems.forEach((item, index) => {
                if (item.separatorBefore) {
                    addSeparator();
                }
                items.push({
                    // TODO: better checkboxes
                    text:
                        item.caption + (item.checked ? ` (${CHECKMARK})` : ``),
                    onClick: async () =>
                        this.instance?.run_context_menu_callback(index),
                    enabled: item.enabled,
                });
            });

            addSeparator();
        }

        if (this.fullscreenEnabled) {
            if (this.isFullscreen) {
                items.push({
                    text: text("context-menu-exit-fullscreen"),
                    onClick: async () => this.setFullscreen(false),
                });
            } else {
                items.push({
                    text: text("context-menu-enter-fullscreen"),
                    onClick: async () => this.setFullscreen(true),
                });
            }
        }

        items.push({
            text: text("context-menu-volume-controls"),
            onClick: async () => {
                this.openVolumeControls();
            },
        });

        if (
            this.instance &&
            this.swfUrl &&
            this.loadedConfig &&
            this.loadedConfig.showSwfDownload === true
        ) {
            addSeparator();
            items.push({
                text: text("context-menu-download-swf"),
                onClick: this.downloadSwf.bind(this),
            });
        }

        if (navigator.clipboard && window.isSecureContext) {
            items.push({
                text: text("context-menu-copy-debug-info"),
                onClick: () =>
                    navigator.clipboard.writeText(this.getPanicData()),
            });
        }

        if (this.checkSaves()) {
            items.push({
                text: text("context-menu-open-save-manager"),
                onClick: this.openSaveManager.bind(this),
            });
        }

        addSeparator();

        items.push({
            text: text("context-menu-about-ruffle", {
                flavor: isExtension ? "extension" : "",
                version: buildInfo.versionName,
            }),
            async onClick() {
                window.open(RUFFLE_ORIGIN, "_blank");
            },
        });
        // Give option to disable context menu when touch support is being used
        // to avoid a long press triggering the context menu. (#1972)
        if (this.isTouch) {
            addSeparator();
            items.push({
                text: text("context-menu-hide"),
                onClick: async () => {
                    this.contextMenuForceDisabled = true;
                },
            });
        }
        return items;
    }

    private pointerDown(event: PointerEvent) {
        this.pointerDownPosition = new Point(event.pageX, event.pageY);
        this.pointerMoveMaxDistance = 0;
        this.startLongPressTimer();
    }

    private clearLongPressTimer(): void {
        if (this.longPressTimer) {
            clearTimeout(this.longPressTimer);
            this.longPressTimer = null;
        }
    }

    private startLongPressTimer(): void {
        const longPressTimeout = 800;
        this.clearLongPressTimer();
        this.longPressTimer = setTimeout(
            () => this.clearLongPressTimer(),
            longPressTimeout,
        );
    }

    private checkLongPressMovement(event: PointerEvent): void {
        if (this.pointerDownPosition !== null) {
            const currentPosition = new Point(event.pageX, event.pageY);
            const distance =
                this.pointerDownPosition.distanceTo(currentPosition);
            if (distance > this.pointerMoveMaxDistance) {
                this.pointerMoveMaxDistance = distance;
            }
        }
    }

    private checkLongPress(event: PointerEvent): void {
        const maxAllowedDistance = 15;
        if (this.longPressTimer) {
            this.clearLongPressTimer();
            // The pointerType condition is to ensure right-click does not trigger
            // a context menu the wrong way the first time you right-click,
            // before contextMenuSupported is set.
        } else if (
            !this.contextMenuSupported &&
            event.pointerType !== "mouse" &&
            this.pointerMoveMaxDistance < maxAllowedDistance
        ) {
            // TODO Implement handling right clicks for mobile.
            this.showContextMenu(event);
        }
    }

    protected suppressContextMenu(): void {
        this._suppressContextMenu = true;
    }

    private showContextMenu(event: MouseEvent | PointerEvent): void {
        if (this.panicked) {
            return;
        }

        event.preventDefault();

        if (this._suppressContextMenu) {
            this._suppressContextMenu = false;
            return;
        }

        if (this.shadow.querySelectorAll(".modal:not(.hidden)").length !== 0) {
            return;
        }

        // TODO Currently when opening context menu,
        //   mouse up event is fired on mouse up,
        //   but should be on context menu close.
        if (event.type === "contextmenu") {
            this.contextMenuSupported = true;
            document.documentElement.addEventListener(
                "click",
                this.hideContextMenu.bind(this),
                {
                    once: true,
                },
            );
        } else {
            document.documentElement.addEventListener(
                "pointerup",
                this.hideContextMenu.bind(this),
                { once: true },
            );
            event.stopPropagation();
        }

        if (
            [false, ContextMenu.Off].includes(
                this.loadedConfig?.contextMenu ?? ContextMenu.On,
            ) ||
            (this.isTouch &&
                this.loadedConfig?.contextMenu ===
                    ContextMenu.RightClickOnly) ||
            this.contextMenuForceDisabled
        ) {
            return;
        }

        // Clear all context menu items.
        while (this.contextMenuElement.firstChild) {
            this.contextMenuElement.removeChild(
                this.contextMenuElement.firstChild,
            );
        }

        // Populate context menu items.
        for (const item of this.contextMenuItems()) {
            if (item === null) {
                this.contextMenuElement.appendChild(
                    <li class="menu-separator">
                        <hr />
                    </li>,
                );
            } else {
                const { text, onClick, enabled } = item;

                const menuItem = (
                    <li
                        class={{
                            "menu-item": true,
                            disabled: enabled === false,
                        }}
                        data-text={text}
                    >
                        {text}
                    </li>
                ) as HTMLElement;
                this.contextMenuElement.appendChild(menuItem);

                if (enabled !== false) {
                    const itemAction = async (event: MouseEvent) => {
                        // Prevent right-clicks from displaying the browser context menu.
                        event.preventDefault();

                        // Prevent the menu from being destroyed.
                        // It's required when we're dealing with async callbacks,
                        // as the async callback may still use the menu in the future.
                        event.stopPropagation();

                        await onClick(event);

                        // Then we have to close the context menu manually after the callback finishes.
                        this.hideContextMenu();
                    };
                    if (this.contextMenuSupported) {
                        menuItem.addEventListener("click", itemAction);
                        menuItem.addEventListener("contextmenu", itemAction);
                    } else {
                        menuItem.addEventListener("pointerup", itemAction);
                    }
                }
            }
        }

        this.contextMenuOverlay.classList.remove("hidden");

        const playerRect = this.element.getBoundingClientRect();
        const contextMenuRect = this.contextMenuElement.getBoundingClientRect();

        // Keep the entire context menu inside the viewport.
        // TODO: Allow the context menu to escape the document body while being mindful of scrollbars.
        const overflowX = Math.max(
            0,
            event.clientX +
                contextMenuRect.width -
                document.documentElement.clientWidth,
        );
        const overflowY = Math.max(
            0,
            event.clientY +
                contextMenuRect.height -
                document.documentElement.clientHeight,
        );
        const x = event.clientX - playerRect.x - overflowX;
        const y = event.clientY - playerRect.y - overflowY;

        this.contextMenuElement.style.transform = `translate(${x}px, ${y}px)`;
    }

    private hideContextMenu(): void {
        this.instance?.clear_custom_menu_items();
        this.contextMenuOverlay.classList.add("hidden");
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
            this.playButton.style.display = "block";
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
            this.unmuteOverlay.style.display = "none";
        }
    }

    /**
     * Plays a silent sound based on the AudioContext's sample rate.
     *
     * This is used to unmute audio on iOS and iPadOS when silent mode is enabled on the device (issue 1552).
     */
    private unmuteAudioContext(): void {
        // No need to play the dummy sound again once audio is unmuted.
        if (isAudioContextUnmuted) {
            return;
        }

        // TODO: Use `navigator.userAgentData` to detect the platform when support improves?
        if (navigator.maxTouchPoints < 1) {
            isAudioContextUnmuted = true;
            return;
        }

        this.container.addEventListener(
            "click",
            () => {
                if (isAudioContextUnmuted) {
                    return;
                }

                const audioContext = this.instance?.audio_context();
                if (!audioContext) {
                    return;
                }

                const audio = new Audio();
                audio.src = (() => {
                    // Returns a seven samples long 8 bit mono WAVE file.
                    // This is required to prevent the AudioContext from desyncing and crashing.
                    const arrayBuffer = new ArrayBuffer(10);
                    const dataView = new DataView(arrayBuffer);
                    const sampleRate = audioContext.sampleRate;
                    dataView.setUint32(0, sampleRate, true);
                    dataView.setUint32(4, sampleRate, true);
                    dataView.setUint16(8, 1, true);
                    const missingCharacters = window
                        .btoa(
                            String.fromCharCode(...new Uint8Array(arrayBuffer)),
                        )
                        .slice(0, 13);
                    return `data:audio/wav;base64,UklGRisAAABXQVZFZm10IBAAAAABAAEA${missingCharacters}AgAZGF0YQcAAACAgICAgICAAAA=`;
                })();

                audio.load();
                audio
                    .play()
                    .then(() => {
                        isAudioContextUnmuted = true;
                    })
                    .catch((err) => {
                        console.warn(`Failed to play dummy sound: ${err}`);
                    });
            },
            { once: true },
        );
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
        attribute: string,
    ): string | null {
        if (attribute) {
            const match = attribute.match(DIMENSION_REGEX);
            if (match) {
                let out = match[1]!;
                if (!match[3]) {
                    // Unitless -- add px for CSS.
                    out += "px";
                }
                return out;
            }
        }
        return null;
    }

    public callExternalInterface(name: string, args: any[]) {
        return this.instance?.call_exposed_callback(name, args);
    }

    protected getObjectId(): string | null {
        return this.element.getAttribute("name");
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
     * Get data included in any panic of this ruffle-player
     *
     * @returns A string containing all the data included in the panic.
     */
    private getPanicData(): string {
        let result = "\n# Player Info\n";
        result += `Allows script access: ${
            this.loadedConfig ? this.loadedConfig.allowScriptAccess : false
        }\n`;
        result += `${this.rendererDebugInfo}\n`;
        result += this.debugPlayerInfo();

        result += "\n# Page Info\n";
        result += `Page URL: ${document.location.href}\n`;
        if (this.swfUrl) {
            result += `SWF URL: ${this.swfUrl}\n`;
        }

        result += "\n# Browser Info\n";
        result += `User Agent: ${window.navigator.userAgent}\n`;
        result += `Platform: ${window.navigator.platform}\n`;
        result += `Has touch support: ${window.navigator.maxTouchPoints > 0}\n`;

        result += "\n# Ruffle Info\n";
        result += `Version: ${buildInfo.versionNumber}\n`;
        result += `Name: ${buildInfo.versionName}\n`;
        result += `Channel: ${buildInfo.versionChannel}\n`;
        result += `Built: ${buildInfo.buildDate}\n`;
        result += `Commit: ${buildInfo.commitHash}\n`;
        result += `Is extension: ${isExtension}\n`;

        result += "\n# Metadata\n";
        if (this.metadata) {
            for (const [key, value] of Object.entries(this.metadata)) {
                result += `${key}: ${value}\n`;
            }
        }

        return result;
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
    protected panic(error: Error | null): void {
        if (this.panicked) {
            // Only show the first major error, not any repeats - they aren't as important
            return;
        }
        this.panicked = true;
        this.hideSplashScreen();
        const originalError = error;

        if (
            error instanceof Error &&
            (error.name === "AbortError" ||
                error.message.includes("AbortError"))
        ) {
            // Firefox: Don't display the panic screen if the user leaves the page while something is still loading
            return;
        } else if (error instanceof LoadRuffleWasmError) {
            const openInNewTab = this.loadedConfig?.openInNewTab;
            const swfUrl =
                this.loadedConfig && "url" in this.loadedConfig
                    ? new URL(this.loadedConfig.url, document.baseURI)
                    : undefined;
            if (openInNewTab && swfUrl) {
                // If it is possible to open the SWF in a new tab offer that option if the WASM failed to load
                this.addOpenInNewTabMessage(openInNewTab, swfUrl);
                return;
            }
            error = error.cause;
        }

        const errorArray: Array<string | null> & {
            stackIndex: number;
            avmStackIndex: number;
        } = Object.assign([], {
            stackIndex: -1,
            avmStackIndex: -1,
        });

        errorArray.push("# Error Info\n");

        if (error instanceof Error) {
            errorArray.push(`Error name: ${error.name}\n`);
            errorArray.push(`Error message: ${error.message}\n`);
            if (error.stack) {
                const stackIndex =
                    errorArray.push(
                        `Error stack:\n\`\`\`\n${error.stack}\n\`\`\`\n`,
                    ) - 1;
                if (error.avmStack) {
                    const avmStackIndex =
                        errorArray.push(
                            `AVM2 stack:\n\`\`\`\n    ${error.avmStack
                                .trim()
                                .replace(/\t/g, "    ")}\n\`\`\`\n`,
                        ) - 1;
                    errorArray.avmStackIndex = avmStackIndex;
                }
                errorArray.stackIndex = stackIndex;
            }
        } else {
            errorArray.push(`Error: ${error}\n`);
        }

        errorArray.push(this.getPanicData());

        // Clears out any existing content (ie play button or canvas) and replaces it with the error screen
        showPanicScreen(this.container, originalError, errorArray, this.swfUrl);

        // Do this last, just in case it causes any cascading issues.
        this.destroy();
    }

    private addOpenInNewTabMessage(
        openInNewTab: (swf: URL) => void,
        swfUrl: URL,
    ) {
        const url = new URL(swfUrl);
        if (this.loadedConfig?.parameters) {
            const parameters = sanitizeParameters(
                this.loadedConfig?.parameters,
            );
            Object.entries(parameters).forEach(([key, value]) => {
                url.searchParams.set(key, value);
            });
        }
        this.hideSplashScreen();

        const div = document.createElement("div");
        div.id = "message-overlay";
        const innerDiv = document.createElement("div");
        innerDiv.className = "message";
        innerDiv.appendChild(textAsParagraphs("message-cant-embed"));

        const buttonDiv = document.createElement("div");
        const link = document.createElement("a");
        link.innerText = text("open-in-new-tab");
        link.onclick = () => openInNewTab(url);
        buttonDiv.appendChild(link);

        innerDiv.appendChild(buttonDiv);
        div.appendChild(innerDiv);
        this.container.prepend(div);
    }

    protected displayRootMovieDownloadFailedMessage(invalidSwf: boolean): void {
        const openInNewTab = this.loadedConfig?.openInNewTab;
        if (
            openInNewTab &&
            this.swfUrl &&
            window.location.origin !== this.swfUrl.origin
        ) {
            this.addOpenInNewTabMessage(openInNewTab, this.swfUrl);
        } else {
            const error = invalidSwf
                ? new InvalidSwfError(this.swfUrl)
                : new LoadSwfError(this.swfUrl);
            this.panic(error);
        }
    }

    /**
     * Show a dismissible message in front of the player.
     *
     * @param message The message shown to the user.
     */
    public displayMessage(message: string): void {
        const div = document.createElement("div");
        div.id = "message-overlay";
        const messageDiv = document.createElement("div");
        messageDiv.className = "message";
        const messageP = document.createElement("p");
        messageP.textContent = message;
        messageDiv.appendChild(messageP);
        const buttonDiv = document.createElement("div");
        const continueButton = document.createElement("button");
        continueButton.id = "continue-btn";
        continueButton.textContent = text("continue");
        buttonDiv.appendChild(continueButton);
        messageDiv.appendChild(buttonDiv);
        div.appendChild(messageDiv);
        this.container.prepend(div);
        (
            this.container.querySelector("#continue-btn") as HTMLButtonElement
        ).onclick = () => {
            div.parentNode!.removeChild(div);
        };
    }

    /**
     * Show a video that uses an unsupported codec in a pop up.
     *
     * @param url The url of the video to be shown over the canvas.
     */
    protected displayUnsupportedVideo(url: string): void {
        const videoHolder = this.videoModal.querySelector("#video-holder");
        if (videoHolder) {
            const video = document.createElement("video");
            video.addEventListener("contextmenu", (event) =>
                event.stopPropagation(),
            );
            video.src = url;
            video.autoplay = true;
            video.controls = true;
            videoHolder.textContent = "";
            videoHolder.appendChild(video);
            this.videoModal.classList.remove("hidden");
        }
    }

    protected displayClipboardModal(accessDenied: boolean): void {
        const description = this.clipboardModal.querySelector(
            "#clipboard-modal-description",
        );
        if (description) {
            description.textContent = text("clipboard-message-description", {
                variant: accessDenied ? "access-denied" : "unsupported",
            });
            this.clipboardModal.classList.remove("hidden");
        }
    }

    private hideSplashScreen(): void {
        this.splashScreen.classList.add("hidden");
        this.container.classList.remove("hidden");
    }

    private showSplashScreen(): void {
        this.splashScreen.classList.remove("hidden");
        this.container.classList.add("hidden");
    }

    protected setMetadata(metadata: MovieMetadata) {
        this.metadata = metadata;
        // TODO: Switch this to ReadyState.Loading when we have streaming support.
        this._readyState = ReadyState.Loaded;
        this.hideSplashScreen();
        this.element.dispatchEvent(
            new CustomEvent(InnerPlayer.LOADED_METADATA),
        );
        // TODO: Move this to whatever function changes the ReadyState to Loaded when we have streaming support.
        this.element.dispatchEvent(new CustomEvent(InnerPlayer.LOADED_DATA));
    }

    avmOutputJson(switcher: number): void {
        if (this.instance) {
            this.instance.avm_output_json(switcher);
        }
    }

    avmOutputJsonCode(code: number): void {
        if (this.instance) {
            this.instance.avm_output_json_code(code);
        }
    }
}

/**
 * Describes the loading state of an SWF movie.
 */
export enum ReadyState {
    /**
     * No movie is loaded, or no information is yet available about the movie.
     */
    HaveNothing = 0,

    /**
     * The movie is still loading, but it has started playback, and metadata is available.
     */
    Loading = 1,

    /**
     * The movie has completely loaded.
     */
    Loaded = 2,
}

/**
 * The volume controls of the Ruffle web GUI.
 */
class VolumeControls {
    isMuted: boolean;
    volume: number;

    constructor(isMuted: boolean, volume: number) {
        this.isMuted = isMuted;
        this.volume = volume;
    }

    /**
     * Returns the volume between 0 and 1 (calculated out of the
     * checkbox and the slider).
     *
     * @returns The volume between 0 and 1.
     */
    public get_volume(): number {
        return !this.isMuted ? this.volume / 100 : 0;
    }
}

/**
 * Returns the URLLoadOptions that have been provided for a specific movie.
 *
 * The function getOptionString is given as an argument and used to get values of configuration
 * options that have been overwritten for this specific movie.
 *
 * The returned URLLoadOptions interface only contains values for the configuration options
 * that have been overwritten for the movie and no default values.
 * This is necessary because any default values would overwrite other configuration
 * settings with a lower priority (e.g. the general RufflePlayer config).
 *
 * @param url The url of the movie.
 * @param getOptionString A function that takes the name of a configuration option.
 * If that configuration option has been overwritten for this specific movie, it returns that value.
 * Otherwise, it returns null.
 * @returns The URLLoadOptions for the movie.
 */
export function getPolyfillOptions(
    url: string,
    getOptionString: (optionName: string) => string | null,
): URLLoadOptions {
    const options: URLLoadOptions = { url };

    const allowNetworking = getOptionString("allowNetworking");
    if (allowNetworking !== null) {
        options.allowNetworking = allowNetworking as NetworkingAccessMode;
    }
    const allowScriptAccess = parseAllowScriptAccess(
        getOptionString("allowScriptAccess"),
        url,
    );
    if (allowScriptAccess !== null) {
        options.allowScriptAccess = allowScriptAccess;
    }
    const backgroundColor = getOptionString("bgcolor");
    if (backgroundColor !== null) {
        options.backgroundColor = backgroundColor;
    }
    const base = getOptionString("base");
    if (base !== null) {
        // "." tells Flash Player to load relative URLs from the SWF's directory
        // All other base values are evaluated relative to the page URL
        if (base === ".") {
            const swfUrl = new URL(url, document.baseURI);
            options.base = new URL(base, swfUrl).href;
        } else {
            options.base = base;
        }
    }
    const menu = parseBoolean(getOptionString("menu"));
    if (menu !== null) {
        options.menu = menu;
    }
    const allowFullscreen = parseBoolean(getOptionString("allowFullScreen"));
    if (allowFullscreen !== null) {
        options.allowFullscreen = allowFullscreen;
    }
    const parameters = getOptionString("flashvars");
    if (parameters !== null) {
        options.parameters = parameters;
    }
    const quality = getOptionString("quality");
    if (quality !== null) {
        options.quality = quality;
    }
    const salign = getOptionString("salign");
    if (salign !== null) {
        options.salign = salign;
    }
    const scale = getOptionString("scale");
    if (scale !== null) {
        options.scale = scale;
    }
    const wmode = getOptionString("wmode");
    if (wmode !== null) {
        options.wmode = wmode as WindowMode;
    }

    return options;
}

/**
 * Returns whether the given filename is a Youtube Flash source.
 *
 * @param filename The filename to test.
 * @returns True if the filename is a Youtube Flash source.
 */
export function isYoutubeFlashSource(filename: string | null): boolean {
    if (filename) {
        let pathname = "";
        let hostname = "";
        try {
            // A base URL is required if `filename` is a relative URL, but we don't need to detect the real URL origin.
            const url = new URL(filename, RUFFLE_ORIGIN);
            pathname = url.pathname;
            hostname = url.hostname;
        } catch (err) {
            // Some invalid filenames, like `///`, could raise a TypeError. Let's fail silently in this situation.
        }
        // See https://wiki.mozilla.org/QA/Youtube_Embedded_Rewrite
        if (
            pathname.startsWith("/v/") &&
            /^(?:(?:www\.|m\.)?youtube(?:-nocookie)?\.com)|(?:youtu\.be)$/i.test(
                hostname,
            )
        ) {
            return true;
        }
    }
    return false;
}

/**
 * Workaround Youtube mixed content if upgradeToHttps is true.
 *
 * @param elem The element to change.
 * @param attr The attribute to adjust.
 */
export function workaroundYoutubeMixedContent(
    elem: Element,
    attr: string,
): void {
    const value = elem.getAttribute(attr);
    const config = window.RufflePlayer?.config ?? {};
    if (value) {
        try {
            const url = new URL(value);
            if (
                url.protocol === "http:" &&
                window.location.protocol === "https:" &&
                (!("upgradeToHttps" in config) ||
                    config.upgradeToHttps !== false)
            ) {
                url.protocol = "https:";
                elem.setAttribute(attr, url.toString());
            }
        } catch (err) {
            // Some invalid filenames, like `///`, could raise a TypeError. Let's fail silently in this situation.
        }
    }
}

/**
 * Determine if an element is a child of a node that was not supported
 * in non-HTML5 compliant browsers. If so, the element was meant to be
 * used as a fallback content.
 *
 * @param elem The element to test.
 * @returns True if the element is inside an <audio> or <video> node.
 */
export function isFallbackElement(elem: Element): boolean {
    let parent = elem.parentElement;
    while (parent !== null) {
        switch (parent.tagName) {
            case "AUDIO":
            case "VIDEO":
                return true;
        }
        parent = parent.parentElement;
    }
    return false;
}

/**
 * Parses a given string or null value to a boolean or null and returns it.
 *
 * @param value The string or null value that should be parsed to a boolean or null.
 * @returns The string as a boolean, if it exists and contains a boolean, otherwise null.
 */
function parseBoolean(value: string | null): boolean | null {
    switch (value?.toLowerCase()) {
        case "true":
            return true;
        case "false":
            return false;
        default:
            return null;
    }
}

/**
 * Parses a string with script access options or null and returns whether the script
 * access options allow the SWF file with the given URL to call JavaScript code in
 * the surrounding HTML file if they exist correctly, otherwise null.
 *
 * @param access The string with the script access options or null.
 * @param url The URL of the SWF file.
 * @returns Whether the script access options allow the SWF file with the given URL to
 * call JavaScript code in the surrounding HTML file if they exist correctly, otherwise null.
 */
function parseAllowScriptAccess(
    access: string | null,
    url: string,
): boolean | null {
    switch (access?.toLowerCase()) {
        case "always":
            return true;
        case "never":
            return false;
        case "samedomain":
            try {
                return (
                    new URL(window.location.href).origin ===
                    new URL(url, window.location.href).origin
                );
            } catch {
                return false;
            }
        default:
            return null;
    }
}
