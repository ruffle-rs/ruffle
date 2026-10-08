function ContextMenu(callbackFunction) {
    // Assignment order is observable via property enumeration.

    // onSelect is assigned even when no callback is given, and is not coerced.
    this.onSelect = callbackFunction;

    this.builtInItems = {
        save: true,
        zoom: true,
        quality: true,
        play: true,
        loop: true,
        rewind: true,
        forward_back: true,
        print: true
    };

    // Construct through global `Array`, which can be overridden.
    this.customItems = new Array();
}

var o = ContextMenu.prototype;

o.copy = function() {
    // Construct through global `ContextMenu`, which can be overridden.
    var copy = new ContextMenu();

    copy.onSelect = this.onSelect;

    // builtInItems is shared rather than copied, so changes to it are visible in both menus.
    copy.builtInItems = this.builtInItems;

    copy.customItems = new Array();
    // this.customItems is looked up on every iteration, which is observable via getters.
    for (var i = 0; i < this.customItems.length; i++) {
        // Dispatch through `push` and each item's `copy`, which can be overridden.
        copy.customItems.push(this.customItems[i].copy());
    }

    return copy;
};

o.hideBuiltInItems = function() {
    // builtInItems is replaced rather than modified, so other references to the old object are unaffected.
    this.builtInItems = {
        save: false,
        zoom: false,
        quality: false,
        play: false,
        loop: false,
        rewind: false,
        forward_back: false,
        print: false
    };
};

// DONT_ENUM, DONT_DELETE, and only visible to SWF 7 and later.
ASSetPropFlags(o, null, 1027);
