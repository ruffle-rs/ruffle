function ContextMenuItem(caption, callbackFunction, separatorBefore, enabled, visible) {
    // Assignment order is observable via property enumeration.
    // None of the arguments are coerced.

    this.caption = caption;
    this.onSelect = callbackFunction;

    // Both `undefined` and `null` select the default value.
    this.separatorBefore = separatorBefore == null ? false : separatorBefore;
    this.enabled = enabled == null ? true : enabled;
    this.visible = visible == null ? true : visible;
}

var o = ContextMenuItem.prototype;

o.copy = function() {
    // Construct through global `ContextMenuItem`, which can be overridden.
    // No arguments are passed, so the copy's properties are first set to their default values.
    var copy = new ContextMenuItem();
    copy.caption = this.caption;
    copy.onSelect = this.onSelect;
    copy.separatorBefore = this.separatorBefore;
    copy.enabled = this.enabled;
    copy.visible = this.visible;
    return copy;
};

// DONT_ENUM, DONT_DELETE, and only visible to SWF 7 and later.
ASSetPropFlags(o, null, 1027);
