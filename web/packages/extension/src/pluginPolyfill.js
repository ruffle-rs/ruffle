class RuffleMimeType {
    constructor(type, description, suffixes) {
        this.type = type;
        this.description = description;
        this.suffixes = suffixes;
    }
}

class RuffleMimeTypeArray {
    constructor(mimeTypes) {
        this._mimeTypes = [];
        this._mimeTypesByType = {};
        for (const mimeType of mimeTypes) {
            this.install(mimeType);
        }
    }

    install(mimeType) {
        this[this._mimeTypes.length] = mimeType;
        this[mimeType.type] = mimeType;
        this._mimeTypes.push(mimeType);
        this._mimeTypesByType[mimeType.type] = mimeType;
    }

    item(index) {
        return this._mimeTypes[index];
    }

    namedItem(type) {
        return this._mimeTypesByType[type];
    }

    get length() {
        return this._mimeTypes.length;
    }
}

class RufflePlugin extends RuffleMimeTypeArray {
    constructor(name, description, filename, mimeTypes) {
        super(mimeTypes);
        this.name = name;
        this.description = description;
        this.filename = filename;
    }

    install(mimeType) {
        if (!mimeType.enabledPlugin) {
            mimeType.enabledPlugin = this;
        }
        super.install(mimeType);
    }
}

class RufflePluginArray {
    constructor(plugins) {
        this._plugins = [];
        this._pluginsByName = {};
        for (const plugin of plugins) {
            this.install(plugin);
        }
    }

    install(plugin) {
        this[this._plugins.length] = plugin;
        this[plugin.name] = plugin;
        this._plugins.push(plugin);
        this._pluginsByName[plugin.name] = plugin;
    }

    item(index) {
        return this._plugins[index];
    }

    namedItem(name) {
        return this._pluginsByName[name];
    }

    get length() {
        return this._plugins.length;
    }
}

function installPlugin(plugin) {
    if (navigator.plugins.install) {
        navigator.plugins.install(plugin);
    } else {
        Object.defineProperty(navigator, "plugins", {
            value: new RufflePluginArray(navigator.plugins),
            writable: false,
        });
        if (plugin.length > 0) {
            Object.defineProperty(navigator, "mimeTypes", {
                value: new RuffleMimeTypeArray(navigator.mimeTypes),
                writable: false,
            });
        }
    }

    for (let i = 0; i < plugin.length; i++) {
        navigator.mimeTypes.install(plugin[i]);
    }
}

installPlugin(new RufflePlugin("Shockwave Flash", "Shockwave Flash 32.0 r0", "ruffle.js", [
    new RuffleMimeType("application/futuresplash", "Shockwave Flash", "spl"),
    new RuffleMimeType("application/x-shockwave-flash", "Shockwave Flash", "swf"),
    new RuffleMimeType("application/x-shockwave-flash2-preview", "Shockwave Flash", "swf"),
    new RuffleMimeType("application/vnd.adobe.flash-movie", "Shockwave Flash", "swf"),
]));
