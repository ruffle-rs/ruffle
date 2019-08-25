/**
 * Equivalent object to `MimeType` that allows us to falsify mime types for
 * plugins.
 */
class RuffleMimeType {
    constructor(type, description, suffixes) {
        this.type = type;
        this.description = description;
        this.suffixes = suffixes;
    }
}

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
class RuffleMimeTypeArray {
    constructor(native_mimetype_array) {
        this.__mimetypes = [];
        this.__named_mimetypes = {};

        for (let mimetype of native_mimetype_array) {
            this.install(mimetype);
        }
    }

    /**
     * Install a MIME Type into the array.
     * 
     * @param {MimeType | RuffleMimeType} mimetype 
     */
    install(mimetype) {
        let id = this.__mimetypes.length;

        this.__mimetypes.push(mimetype);
        this.__named_mimetypes[mimetype.type] = mimetype;
        this[mimetype.type] = mimetype;
        this[id] = mimetype;
    }

    item(index) {
        return this.__mimetypes[index];
    }

    namedItem(name) {
        return this.__named_mimetypes[name];
    }

    get length() {
        return this.__mimetypes.length;
    }
}

/**
 * Equivalent object to `Plugin` that allows us to falsify plugins.
 */
class RufflePlugin extends RuffleMimeTypeArray {
    constructor(name, description, filename, mimetypes) {
        super(mimetypes);

        this.name = name;
        this.description = description;
        this.filename = filename;
    }

    install(mimetype) {
        if (!mimetype.enabledPlugin) {
            mimetype.enabledPlugin = this;
        }

        super.install(mimetype);
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
    constructor(native_plugin_array) {
        this.__plugins = [];
        this.__named_plugins = {};

        for (let plugin of native_plugin_array) {
            this.install(plugin);
        }
    }

    install(plugin) {
        let id = this.__plugins.length;

        this.__plugins.push(plugin);
        this.__named_plugins[plugin.name] = plugin;
        this[plugin.name] = plugin;
        this[id] = plugin;
    }

    item(index) {
        return this.__plugins[index];
    }

    namedItem(name) {
        return this.__named_plugins[name];
    }

    get length() {
        return this.__plugins.length;
    }
};

export const FLASH_PLUGIN = new RufflePlugin("Shockwave Flash", "Shockwave Flash (compatible; Ruffle 0)", "ruffle.js", [
    new RuffleMimeType("application/futuresplash", "Shockwave Flash", "spl"),
    new RuffleMimeType("application/x-shockwave-flash", "Shockwave Flash", "spl"),
]);

/**
 * Install a fake plugin such that detectors will see it in `navigator.plugins`.
 * 
 * This function takes care to check if the existing implementation of
 * `navigator.plugins` already accepts fake plugin entries. If so, it will use
 * that version of the plugin array. This allows the plugin polyfill to compose
 * across multiple plugin emulators with the first emulator's polyfill winning.
 */
export function install_plugin(plugin) {
    if (!navigator.plugins.install) {
        Object.defineProperty(navigator, "plugins", {
            value: new RufflePluginArray(navigator.plugins),
            writable: false
        });
    }

    navigator.plugins.install(plugin);

    if (plugin.length > 0 && !navigator.mimeTypes.install) {
        Object.defineProperty(navigator, "mimeTypes", {
            value: new RuffleMimeTypeArray(navigator.mimeTypes),
            writable: false
        });
    }

    for (var i = 0; i < plugin.length; i += 1) {
        navigator.mimeTypes.install(plugin[i]);
    }
}