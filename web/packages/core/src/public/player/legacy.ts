import { DataLoadOptions, URLLoadOptions } from "../config";
import { MovieMetadata } from "./movie-metadata";
import { ReadyState } from "../../internal/player/inner";

/**
 * Legacy interface to the Ruffle API.
 *
 * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
 * Any of these methods or properties may be replaced by Flash and are not guaranteed to exist.
 */
export interface LegacyRuffleAPI {
    /**
     * A movie can communicate with the hosting page using fscommand
     * as long as script access is allowed.
     *
     * @param command A string passed to the host application for any use.
     * @param args A string passed to the host application for any use.
     * @returns True if the command was handled.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.onFSCommand}
     */
    onFSCommand: ((command: string, args: string) => boolean) | null;

    /**
     * Any configuration that should apply to this specific player.
     * This will be defaulted with any global configuration.
     *
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.config}
     */
    config: URLLoadOptions | DataLoadOptions | object;

    /**
     * The effective config loaded with the last call to `load()`.
     * If no such call has been made, this will be `null`.
     *
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.loadedConfig}
     */
    readonly loadedConfig: URLLoadOptions | DataLoadOptions | null;

    /**
     * Indicates the readiness of the playing movie.
     *
     * @returns The `ReadyState` of the player.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.readyState}
     */
    get readyState(): ReadyState;

    /**
     * The metadata of the playing movie (such as movie width and height).
     * These are inherent properties stored in the SWF file and are not affected by runtime changes.
     * For example, `metadata.width` is the width of the SWF file, and not the width of the Ruffle player.
     *
     * @returns The metadata of the movie, or `null` if the movie metadata has not yet loaded.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.metadata}
     */
    get metadata(): MovieMetadata | null;

    /**
     * Reloads the player, as if you called {@link load} with the same config as the last time it was called.
     *
     * If this player has never been loaded, this method will return an error.
     *
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.reload}
     */
    reload(): Promise<void>;

    /**
     * Loads a specified movie into this player.
     *
     * This will replace any existing movie that may be playing.
     *
     * @param options One of the following:
     * - A URL, passed as a string, which will load a URL with default options.
     * - A {@link URLLoadOptions} object, to load a URL with options.
     * - A {@link DataLoadOptions} object, to load data with options.
     * The options, if provided, must only contain values provided for this specific movie.
     * They must not contain any default values, since those would overwrite other configuration
     * settings with a lower priority (e.g. the general RufflePlayer config).
     *
     * The options will be defaulted by the {@link config} field, which itself
     * is defaulted by a global `window.RufflePlayer.config`.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.load}
     */
    load(options: string | URLLoadOptions | DataLoadOptions): Promise<void>;

    /**
     * Resumes the movie from suspension.
     *
     * The movie will now resume executing any frames, scripts and sounds.
     * If the movie is not suspended or no movie is loaded, this method will do nothing.
     *
     * @remarks
     * This method was confusingly named and kept for legacy compatibility.
     * "Playing" in this context referred to "not being suspended", and <b>not</b> the Flash concept of playing/paused.
     *
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.resume}
     */
    play(): void;

    /**
     * Checks if this player is not suspended.
     *
     * A suspended movie will not execute any frames, scripts or sounds.
     * This movie is considered inactive and will not wake up until resumed.
     * If no movie is loaded, this method will return true.
     *
     * @remarks
     * This method was confusingly named and kept for legacy compatibility.
     * "Playing" in this context referred to "not being suspended", and <b>not</b> the Flash concept of playing/paused.
     *
     * @returns True if this player is playing, false if it's paused or hasn't started yet.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.suspended} (though inversed!)
     */
    get isPlaying(): boolean;

    /**
     * Returns the master volume of the player.
     *
     * The volume is linear and not adapted for logarithmic hearing.
     *
     * @returns The volume. 1.0 is 100% volume.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.volume}
     */
    get volume(): number;

    /**
     * Sets the master volume of the player.
     *
     * The volume should be linear and not adapted for logarithmic hearing.
     *
     * @param value The volume. 1.0 is 100% volume.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.volume}
     */
    set volume(value: number);

    /**
     * Checks if this player is allowed to be fullscreen by the browser.
     *
     * @returns True if you may call {@link enterFullscreen}.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.fullscreenEnabled}
     */
    get fullscreenEnabled(): boolean;

    /**
     * Checks if this player is currently fullscreen inside the browser.
     *
     * @returns True if it is fullscreen.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.isFullscreen}
     */
    get isFullscreen(): boolean;

    /**
     * Exported function that requests the browser to change the fullscreen state if
     * it is allowed.
     *
     * @param isFull Whether to set to fullscreen or return to normal.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.setFullscreen}
     */
    setFullscreen(isFull: boolean): void;

    /**
     * Requests the browser to make this player fullscreen.
     *
     * This is not guaranteed to succeed, please check {@link fullscreenEnabled} first.
     *
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.enterFullscreen}
     */
    enterFullscreen(): void;

    /**
     * Requests the browser to no longer make this player fullscreen.
     *
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.exitFullscreen}
     */
    exitFullscreen(): void;

    /**
     * Suspends the movie.
     *
     * A suspended movie will not execute any frames, scripts or sounds.
     * This movie is considered inactive and will not wake up until resumed.
     * If the movie is already suspended or no movie is loaded, this method will do nothing.
     *
     * @remarks
     * This method was confusingly named and kept for legacy compatibility.
     * "Pause" in this context referred to "suspended", and <b>not</b> the Flash concept of playing/paused.
     *
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.suspend}
     */
    pause(): void;

    /**
     * Sets a trace observer on this flash player.
     *
     * The observer will be called, as a function, for each message that the playing movie will "trace" (output).
     *
     * @param observer The observer that will be called for each trace.
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.traceObserver}
     */
    set traceObserver(observer: ((message: string) => void) | null);

    /**
     * Fetches the loaded SWF and downloads it.
     *
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.downloadSwf}
     */
    downloadSwf(): Promise<void>;

    /**
     * Show a dismissible message in front of the player.
     *
     * @param message The message shown to the user.
     *
     * @deprecated Please use {@link PlayerElement.ruffle | ruffle()} to access a versioned API.
     * This method may be replaced by Flash and is not guaranteed to exist.
     * A direct replacement is {@link PlayerV1.displayMessage}
     */
    displayMessage(message: string): void;
}
