function keys(obj) {
    var result = "";
    for (var key in obj) {
        result += key + " ";
    }
    return result;
}

function describe(value) {
    if (typeof value == "string") {
        return "\"" + value + "\"";
    }
    if (typeof value == "function") {
        return "[function]";
    }
    return String(value);
}

function dump(obj) {
    for (var key in obj) {
        trace("  " + key + " = " + describe(obj[key]));
    }
}

function addTracingProperty(name, obj, prop, value) {
    obj.addProperty(prop, function() {
        trace("  get " + name + "." + prop);
        return value;
    }, function(newValue) {
        trace("  set " + name + "." + prop + " = " + describe(newValue));
        value = newValue;
    });
}

function callback() {
}

var originalContextMenuItem = ContextMenuItem;

trace("/// Constructors");

trace("// new ContextMenuItem()");
dump(new ContextMenuItem());
trace("");

trace("// new ContextMenuItem('caption', callback, true, false, false)");
var item = new ContextMenuItem("caption", callback, true, false, false);
dump(item);
trace(item.onSelect === callback);
trace("");

var argumentCases = [undefined, null, 0, "", NaN];
var argumentLabels = ["undefined", "null", "0", "''", "NaN"];
for (var i = 0; i < argumentCases.length; i++) {
    var label = argumentLabels[i];
    var value = argumentCases[i];
    trace("// new ContextMenuItem(" + [label, label, label, label, label].join(", ") + ")");
    dump(new ContextMenuItem(value, value, value, value, value));
    trace("");
}

trace("// ContextMenuItem.call(obj, 'caption', callback)");
item = {};
trace(ContextMenuItem.call(item, "caption", callback));
trace(keys(item));
trace("");
trace("");

trace("/// Prototype");

trace("// ContextMenuItem.prototype");
trace(keys(ContextMenuItem.prototype));
trace("");

trace("// ContextMenuItem.prototype, without DONT_ENUM");
ASSetPropFlags(ContextMenuItem.prototype, "copy", 0, 1);
trace(keys(ContextMenuItem.prototype));
ASSetPropFlags(ContextMenuItem.prototype, "copy", 1);
trace("");

trace("// delete ContextMenuItem.prototype.copy");
trace(delete ContextMenuItem.prototype.copy);
trace("");
trace("");

trace("/// copy");

trace("// item.copy()");
item = new ContextMenuItem("caption", callback, true, false, false);
item.extra = true;
var copy = item.copy();
dump(copy);
trace(copy instanceof ContextMenuItem);
trace(copy.onSelect === item.onSelect);
trace("");

trace("// item.copy(), with tracing properties");
item = new ContextMenuItem();
addTracingProperty("item", item, "caption", 5);
addTracingProperty("item", item, "onSelect", "callback");
addTracingProperty("item", item, "separatorBefore", 1);
addTracingProperty("item", item, "enabled", 0);
addTracingProperty("item", item, "visible", "x");
var copiedProperties = ["caption", "onSelect", "separatorBefore", "enabled", "visible"];
_global.ContextMenuItem = function() {
    trace("  new ContextMenuItem(" + arguments.join(", ") + ")");
    for (var j = 0; j < copiedProperties.length; j++) {
        addTracingProperty("copy", this, copiedProperties[j], undefined);
    }
};
item.copy();
_global.ContextMenuItem = originalContextMenuItem;
trace("");

trace("// ContextMenuItem.prototype.copy.call({})");
dump(ContextMenuItem.prototype.copy.call({}));
trace("");
