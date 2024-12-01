import {
    FLASH_MIMETYPE,
    FUTURESPLASH_MIMETYPE,
    FLASH7_AND_8_MIMETYPE,
    FLASH_MOVIE_MIMETYPE,
} from "./flash-identifiers";

/**
 * Replacement object for `MimeTypeArray` that lets us install new fake mime
 * types.
 *
 * Unlike plugins we can at least enumerate mime types in Firefox, so we don't
 * lose data.
 *
 * We also expose a method called `install` which adds a new plugin. This is
 * used to falsify Flash detection. If the existing `navigator.mimeTypes` has an
 * `install` method, you should not use `RuffleMimeTypeArray` as some other
 * plugin emulator is already present.
 */
class RuffleMimeTypeArray implements MimeTypeArray {
    private readonly __mimeTypes: MimeType[];
    private readonly __namedMimeTypes: Record<string, MimeType>;

    constructor(mimeTypes?: MimeTypeArray) {
        this.__mimeTypes = [];
        this.__namedMimeTypes = {};

        if (mimeTypes) {
            for (let i = 0; i < mimeTypes.length; i++) {
                this.install(mimeTypes[i]!);
            }
        }
    }

    /**
     * Install a MIME Type into the array.
     *
     * @param mimeType The mime type to install
     */
    install(mimeType: MimeType): void {
        const wrapper = new RuffleMimeType(mimeType);

        const index = this.__mimeTypes.length;
        this.__mimeTypes.push(wrapper);
        this.__namedMimeTypes[mimeType.type] = wrapper;
        this[wrapper.type] = wrapper;
        this[index] = wrapper;
    }

    item(index: number): MimeType {
        // This behavior is done to emulate a 32-bit uint,
        // which browsers use.
        return this.__mimeTypes[index >>> 0]!;
    }

    namedItem(name: string): MimeType {
        return this.__namedMimeTypes[name]!;
    }

    get length(): number {
        return this.__mimeTypes.length;
    }

    [index: number]: MimeType;

    [name: string]: unknown;

    [Symbol.iterator](): IterableIterator<MimeType> {
        return this.__mimeTypes[Symbol.iterator]();
    }

    get [Symbol.toStringTag](): string {
        return "MimeTypeArray";
    }
}

/**
 * Replacement object for the built-in MimeType object.
 * This only exists, because the built-in type is not constructable and we
 * need to spoof `window.MimeType`.
 */
class RuffleMimeType implements MimeType {
    private readonly __mimeType: MimeType;

    constructor(mimeType: MimeType) {
        this.__mimeType = mimeType;
    }

    get type(): string {
        return this.__mimeType.type;
    }

    get description(): string {
        return this.__mimeType.description;
    }

    get suffixes(): string {
        return this.__mimeType.suffixes;
    }

    get enabledPlugin(): Plugin {
        return this.__mimeType.enabledPlugin;
    }

    get [Symbol.toStringTag](): string {
        return "MimeType";
    }
}

/**
 * Equivalent object to `Plugin` that allows us to falsify plugins.
 */
class RufflePlugin extends RuffleMimeTypeArray implements Plugin {
    constructor(
        readonly name: string,
        readonly description: string,
        readonly filename: string,
    ) {
        super();
    }
}

/**
 * Replacement object for `PluginArray` that lets us install new fake plugins.
 *
 * This object needs to wrap the native plugin array, since the user might have
 * actual plugins installed. Firefox doesn't let us enumerate the array, though,
 * which has some consequences. Namely, we can't actually perfectly wrap the
 * native plugin array, at least unless there's some secret "unresolved object
 * property name handler" that I've never known before in JS...
 *
 * We can still wrap `namedItem` perfectly at least.
 *
 * We also expose a method called `install` which adds a new plugin. This is
 * used to falsify Flash detection. If the existing `navigator.plugins` has an
 * `install` method, you should not use `RufflePluginArray` as some other plugin
 * emulator is already present.
 */
class RufflePluginArray implements PluginArray {
    private readonly __plugins: Plugin[];
    private readonly __namedPlugins: Record<string, Plugin>;

    constructor(plugins: PluginArray) {
        this.__plugins = [];
        this.__namedPlugins = {};

        for (let i = 0; i < plugins.length; i++) {
            this.install(plugins[i]!);
        }
    }

    install(plugin: Plugin): void {
        const index = this.__plugins.length;
        this.__plugins.push(plugin);
        this.__namedPlugins[plugin.name] = plugin;
        this[plugin.name] = plugin;
        this[index] = plugin;
    }

    item(index: number): Plugin {
        // This behavior is done to emulate a 32-bit uint,
        // which browsers use. Cloudflare's anti-bot
        // checks rely on this.
        return this.__plugins[index >>> 0]!;
    }

    namedItem(name: string): Plugin {
        return this.__namedPlugins[name]!;
    }

    refresh(): void {
        // Nothing to do, we just need to define the method.
    }

    [index: number]: Plugin;

    [name: string]: unknown;

    [Symbol.iterator](): IterableIterator<Plugin> {
        return this.__plugins[Symbol.iterator]();
    }

    get [Symbol.toStringTag](): string {
        return "PluginArray";
    }

    get length(): number {
        return this.__plugins.length;
    }
}

/**
 * A fake plugin designed to trigger Flash detection scripts.
 */
export const FLASH_PLUGIN = new RufflePlugin(
    "Shockwave Flash",
    "Shockwave Flash 32.0 r0",
    "ruffle.js",
);

FLASH_PLUGIN.install({
    type: FUTURESPLASH_MIMETYPE,
    description: "Shockwave Flash",
    suffixes: "spl",
    enabledPlugin: FLASH_PLUGIN,
});
FLASH_PLUGIN.install({
    type: FLASH_MIMETYPE,
    description: "Shockwave Flash",
    suffixes: "swf",
    enabledPlugin: FLASH_PLUGIN,
});
FLASH_PLUGIN.install({
    type: FLASH7_AND_8_MIMETYPE,
    description: "Shockwave Flash",
    suffixes: "swf",
    enabledPlugin: FLASH_PLUGIN,
});
FLASH_PLUGIN.install({
    type: FLASH_MOVIE_MIMETYPE,
    description: "Shockwave Flash",
    suffixes: "swf",
    enabledPlugin: FLASH_PLUGIN,
});

declare global {
    interface PluginArray {
        install?: (plugin: Plugin) => void;
    }

    interface MimeTypeArray {
        install?: (mimeType: MimeType) => void;
    }
}

/**
 * Install a fake plugin such that detectors will see it in `navigator.plugins`.
 *
 * This function takes care to check if the existing implementation of
 * `navigator.plugins` already accepts fake plugin entries. If so, it will use
 * that version of the plugin array. This allows the plugin polyfill to compose
 * across multiple plugin emulators with the first emulator's polyfill winning.
 *
 * @param plugin The plugin to install
 */
export function installPlugin(plugin: RufflePlugin): void {
    if (navigator.plugins.namedItem("Shockwave Flash")) {
        return;
    }
    if (!("install" in navigator.plugins) || !navigator.plugins["install"]) {
        Object.defineProperty(window, "PluginArray", {
            value: RufflePluginArray,
        });
        Object.defineProperty(navigator, "plugins", {
            value: new RufflePluginArray(navigator.plugins),
            writable: false,
        });
    }

    const plugins = navigator.plugins;
    plugins.install!(plugin);

    if (
        plugin.length > 0 &&
        (!("install" in navigator.mimeTypes) || !navigator.mimeTypes["install"])
    ) {
        Object.defineProperty(window, "MimeTypeArray", {
            value: RuffleMimeTypeArray,
        });
        Object.defineProperty(window, "MimeType", {
            value: RuffleMimeType,
        });
        Object.defineProperty(navigator, "mimeTypes", {
            value: new RuffleMimeTypeArray(navigator.mimeTypes),
            writable: false,
        });
    }

    const mimeTypes = navigator.mimeTypes;
    for (let i = 0; i < plugin.length; i += 1) {
        mimeTypes.install!(plugin[i]!);
    }
}
