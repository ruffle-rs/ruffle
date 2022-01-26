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
    private readonly __mimetypes: MimeType[];
    private readonly __named_mimetypes: Record<string, MimeType>;

    constructor(native_mimetype_array: MimeTypeArray | null) {
        this.__mimetypes = [];
        this.__named_mimetypes = {};

        if (native_mimetype_array) {
            for (let i = 0; i < native_mimetype_array.length; i++) {
                this.install(native_mimetype_array[i]);
            }
        }
    }

    /**
     * Install a MIME Type into the array.
     *
     * @param mimetype The mimetype to install
     */
    install(mimetype: MimeType): void {
        const id = this.__mimetypes.length;

        this.__mimetypes.push(mimetype);
        this.__named_mimetypes[mimetype.type] = mimetype;
        this[mimetype.type] = mimetype;
        this[id] = mimetype;
    }

    item(index: number): MimeType {
        return this.__mimetypes[index];
    }

    namedItem(name: string): MimeType {
        return this.__named_mimetypes[name];
    }

    get length(): number {
        return this.__mimetypes.length;
    }

    [index: number]: MimeType;

    [name: string]: unknown;

    [Symbol.iterator](): IterableIterator<MimeType> {
        return this.__mimetypes[Symbol.iterator]();
    }
}

/**
 * Equivalent object to `Plugin` that allows us to falsify plugins.
 */
class RufflePlugin extends RuffleMimeTypeArray implements Plugin {
    name: string;
    description: string;
    filename: string;

    constructor(
        name: string,
        description: string,
        filename: string,
        mimetypes: RuffleMimeTypeArray | null
    ) {
        super(mimetypes);

        this.name = name;
        this.description = description;
        this.filename = filename;
    }

    install(mimetype: MimeType): void {
        super.install(mimetype);
    }

    [index: number]: MimeType;

    [Symbol.iterator](): IterableIterator<MimeType> {
        return super[Symbol.iterator]();
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
class RufflePluginArray {
    private readonly __plugins: Plugin[];
    private readonly __named_plugins: Record<string, Plugin>;

    constructor(native_plugin_array: PluginArray) {
        this.__plugins = [];
        this.__named_plugins = {};

        for (let i = 0; i < native_plugin_array.length; i++) {
            this.install(native_plugin_array[i]);
        }
    }

    install(plugin: Plugin): void {
        const id = this.__plugins.length;

        this.__plugins.push(plugin);
        this.__named_plugins[plugin.name] = plugin;
        this[plugin.name] = plugin;
        this[id] = plugin;
    }

    item(index: number): Plugin {
        return this.__plugins[index];
    }

    namedItem(name: string): Plugin {
        return this.__named_plugins[name];
    }

    refresh(): void {
        // Nothing to do, we just need to define the method.
    }

    [index: number]: Plugin;

    [name: string]: unknown;

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
    null
);

FLASH_PLUGIN.install({
    type: "application/futuresplash",
    description: "Shockwave Flash",
    suffixes: "spl",
    enabledPlugin: FLASH_PLUGIN,
});
FLASH_PLUGIN.install({
    type: "application/x-shockwave-flash",
    description: "Shockwave Flash",
    suffixes: "swf",
    enabledPlugin: FLASH_PLUGIN,
});
FLASH_PLUGIN.install({
    type: "application/x-shockwave-flash2-preview",
    description: "Shockwave Flash",
    suffixes: "swf",
    enabledPlugin: FLASH_PLUGIN,
});
FLASH_PLUGIN.install({
    type: "application/vnd.adobe.flash-movie",
    description: "Shockwave Flash",
    suffixes: "swf",
    enabledPlugin: FLASH_PLUGIN,
});

declare global {
    interface PluginArray {
        install?: (plugin: Plugin) => void;
    }
}

declare global {
    interface MimeTypeArray {
        install?: (mime: MimeType) => void;
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
    if (!("install" in navigator.plugins) || !navigator.plugins["install"]) {
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
        Object.defineProperty(navigator, "mimeTypes", {
            value: new RuffleMimeTypeArray(navigator.mimeTypes),
            writable: false,
        });
    }

    const mimeTypes = navigator.mimeTypes;
    for (let i = 0; i < plugin.length; i += 1) {
        mimeTypes.install!(plugin[i]);
    }
}
