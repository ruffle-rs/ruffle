import { DataLoadOptions, URLLoadOptions } from "../load-options";
import { MovieMetadata } from "../movie-metadata";
import { ReadyState } from "../internal/player/inner";

/**
 * Legacy interface to the Ruffle API.
 *
 * These methods are deprecated and only exist for backwards compatibility.
 * Due to the nature of Flash, they may be replaced at any time via ActionScript's `ExternalInterface` class.
 */
export interface LegacyRuffleAPI {
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
    config: URLLoadOptions | DataLoadOptions | object;

    /**
     * The effective config loaded with the last call to `load()`.
     * If no such call has been made, this will be `null`.
     */
    readonly loadedConfig: URLLoadOptions | DataLoadOptions | null;

    /**
     * Indicates the readiness of the playing movie.
     *
     * @returns The `ReadyState` of the player.
     */
    get readyState(): ReadyState;

    /**
     * The metadata of the playing movie (such as movie width and height).
     * These are inherent properties stored in the SWF file and are not affected by runtime changes.
     * For example, `metadata.width` is the width of the SWF file, and not the width of the Ruffle player.
     *
     * @returns The metadata of the movie, or `null` if the movie metadata has not yet loaded.
     */
    get metadata(): MovieMetadata | null;

    /**
     * Reloads the player, as if you called {@link RufflePlayer.load} with the same config as the last time it was called.
     *
     * If this player has never been loaded, this method will return an error.
     */
    reload(): Promise<void>;

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
     *
     * The options will be defaulted by the [[config]] field, which itself
     * is defaulted by a global `window.RufflePlayer.config`.
     */
    load(options: string | URLLoadOptions | DataLoadOptions): Promise<void>;

    /**
     * Plays or resumes the movie.
     */
    play(): void;

    /**
     * Whether this player is currently playing.
     *
     * @returns True if this player is playing, false if it's paused or hasn't started yet.
     */
    get isPlaying(): boolean;

    /**
     * Returns the master volume of the player.
     *
     * The volume is linear and not adapted for logarithmic hearing.
     *
     * @returns The volume. 1.0 is 100% volume.
     */
    get volume(): number;

    /**
     * Sets the master volume of the player.
     *
     * The volume should be linear and not adapted for logarithmic hearing.
     *
     * @param value The volume. 1.0 is 100% volume.
     */
    set volume(value: number);

    /**
     * Checks if this player is allowed to be fullscreen by the browser.
     *
     * @returns True if you may call [[enterFullscreen]].
     */
    get fullscreenEnabled(): boolean;

    /**
     * Checks if this player is currently fullscreen inside the browser.
     *
     * @returns True if it is fullscreen.
     */
    get isFullscreen(): boolean;

    /**
     * Exported function that requests the browser to change the fullscreen state if
     * it is allowed.
     *
     * @param isFull Whether to set to fullscreen or return to normal.
     */
    setFullscreen(isFull: boolean): void;

    /**
     * Requests the browser to make this player fullscreen.
     *
     * This is not guaranteed to succeed, please check [[fullscreenEnabled]] first.
     */
    enterFullscreen(): void;

    /**
     * Requests the browser to no longer make this player fullscreen.
     */
    exitFullscreen(): void;

    /**
     * Pauses this player.
     *
     * No more frames, scripts or sounds will be executed.
     * This movie will be considered inactive and will not wake up until resumed.
     */
    pause(): void;

    /**
     * Sets a trace observer on this flash player.
     *
     * The observer will be called, as a function, for each message that the playing movie will "trace" (output).
     *
     * @param observer The observer that will be called for each trace.
     */
    set traceObserver(observer: ((message: string) => void) | null);

    /**
     * Fetches the loaded SWF and downloads it.
     */
    downloadSwf(): Promise<void>;
}
