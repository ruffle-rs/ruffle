function keys(obj) {
    var result = "";
    for (var key in obj) {
        result += key + " ";
    }
    return result;
}

function dump(obj) {
    for (var key in obj) {
        trace("  " + key + " = " + obj[key]);
    }
}

function addTracingProperty(name, obj, prop, value) {
    obj.addProperty(prop, function() {
        trace("  get " + name + "." + prop);
        return value;
    }, function(newValue) {
        trace("  set " + name + "." + prop + " = " + newValue);
        value = newValue;
    });
}

function tracingMethod(name, obj, method) {
    var original = obj[method];
    obj[method] = function() {
        trace("  call " + name + "." + method + "(" + originalArrayJoin.call(arguments, ", ") + ")");
        return original.apply(this, arguments);
    };
}

function tracingConstructor(name) {
    return function() {
        // `arguments.join` can't be used, as `arguments` inherits from `Array.prototype`, which might be replaced.
        trace("  new " + name + "(" + originalArrayJoin.call(arguments, ", ") + ")");
    };
}

function tracingNumber(name, value) {
    return {valueOf: function() {
        trace("  valueOf " + name);
        return value;
    }};
}

function callback() {
}

function copyItem() {
    return this.name + " copy";
}

var originalContextMenu = ContextMenu;
var originalObject = Object;
var originalArray = Array;
var originalArrayPush = Array.prototype.push;
var originalArrayJoin = Array.prototype.join;

trace("/// Constructors");

trace("// new ContextMenu()");
var menu = new ContextMenu();
dump(menu);
trace("");

trace("// new ContextMenu().builtInItems");
dump(menu.builtInItems);
trace("");

trace("// new ContextMenu().customItems instanceof Array");
trace(menu.customItems instanceof Array);
trace("");

var callbackCases = [callback, undefined, null, 5, "string", {}];
var callbackLabels = ["callback", "undefined", "null", "5", "'string'", "{}"];
for (var i = 0; i < callbackCases.length; i++) {
    var label = callbackLabels[i];
    trace("// new ContextMenu(" + label + ").onSelect === " + label);
    menu = new ContextMenu(callbackCases[i]);
    trace(menu.onSelect === callbackCases[i]);
    trace("");
}

var instanceProperties = ["builtInItems", "customItems"];
menu = new ContextMenu();
var other = new ContextMenu();
for (i = 0; i < instanceProperties.length; i++) {
    var property = instanceProperties[i];
    trace("// new ContextMenu()." + property + " === new ContextMenu()." + property);
    trace(menu[property] === other[property]);
    trace("");
}

trace("// new ContextMenu(), with Object replaced");
_global.Object = tracingConstructor("Object");
menu = new ContextMenu();
_global.Object = originalObject;
trace("");

trace("// new ContextMenu(), with Array replaced");
_global.Array = tracingConstructor("Array");
menu = new ContextMenu();
_global.Array = originalArray;
trace(menu.customItems instanceof Array);
trace("");

trace("// ContextMenu.call(obj, callback)");
menu = {};
trace(ContextMenu.call(menu, callback));
trace(keys(menu));
trace("");
trace("");

trace("/// Prototype");

trace("// ContextMenu.prototype");
trace(keys(ContextMenu.prototype));
trace("");

trace("// ContextMenu.prototype, without DONT_ENUM");
ASSetPropFlags(ContextMenu.prototype, "copy,hideBuiltInItems", 0, 1);
trace(keys(ContextMenu.prototype));
ASSetPropFlags(ContextMenu.prototype, "copy,hideBuiltInItems", 1);
trace("");

var methods = ["copy", "hideBuiltInItems"];
for (i = 0; i < methods.length; i++) {
    trace("// delete ContextMenu.prototype." + methods[i]);
    trace(delete ContextMenu.prototype[methods[i]]);
    trace("");
}
trace("");

trace("/// copy");

menu = new ContextMenu(callback);
var copy = menu.copy();

trace("// menu.copy() instanceof ContextMenu");
trace(copy instanceof ContextMenu);
trace("");

var copiedProperties = ["onSelect", "builtInItems", "customItems"];
for (i = 0; i < copiedProperties.length; i++) {
    property = copiedProperties[i];
    trace("// menu.copy()." + property + " === menu." + property);
    trace(copy[property] === menu[property]);
    trace("");
}

trace("// menu.copy(), with tracing properties");
menu = new ContextMenu(callback);
var customItems = {length: tracingNumber("customItems.length", 2)};
customItems[0] = {name: "item0", copy: copyItem};
customItems[1] = {name: "item1", copy: copyItem};
menu.customItems = customItems;
for (i = 0; i < copiedProperties.length; i++) {
    addTracingProperty("menu", menu, copiedProperties[i], menu[copiedProperties[i]]);
}
_global.ContextMenu = function() {
    trace("  new ContextMenu(" + originalArrayJoin.call(arguments, ", ") + ")");
    for (var j = 0; j < copiedProperties.length; j++) {
        addTracingProperty("copy", this, copiedProperties[j], undefined);
    }
};
tracingMethod("Array.prototype", Array.prototype, "push");
copy = menu.copy();
Array.prototype.push = originalArrayPush;
_global.ContextMenu = originalContextMenu;
trace("");

trace("// menu.copy(), with Array replaced");
menu = new ContextMenu();
_global.Array = tracingConstructor("Array");
copy = menu.copy();
_global.Array = originalArray;
trace(copy.customItems instanceof Array);
trace("");

trace("// ContextMenu.prototype.copy.call({})");
dump(ContextMenu.prototype.copy.call({}));
trace("");
trace("");

trace("/// hideBuiltInItems");

trace("// menu.hideBuiltInItems()");
menu = new ContextMenu();
trace(menu.hideBuiltInItems());
dump(menu.builtInItems);
trace("");

trace("// menu.hideBuiltInItems(), with tracing builtInItems");
menu = new ContextMenu();
addTracingProperty("menu", menu, "builtInItems", menu.builtInItems);
menu.hideBuiltInItems();
trace("");

trace("// menu.hideBuiltInItems(), with Object replaced");
menu = new ContextMenu();
_global.Object = tracingConstructor("Object");
menu.hideBuiltInItems();
_global.Object = originalObject;
trace("");
