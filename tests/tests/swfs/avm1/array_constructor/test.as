trace("// new Array()");
var a = new Array();
trace(a);
trace(a.length);

trace("// new Array(5)");
var a = new Array(5);
trace(a);
trace(a.length);

trace('// new Array("3")');
var a = new Array("3");
trace(a);
trace(a.length);

trace("// new Array(o)");
var o = { valueOf: function() { return 3; } };
var a = new Array(o);
trace(a);
trace(a.length);

trace("// new Array(-1)");
var a = new Array(-1);
trace(a);
trace(a.length);

trace("// new Array(undefined)");
var a = new Array(undefined);
trace(a);
trace(a.length);

trace("// new Array(null)");
var a = new Array(null);
trace(a);
trace(a.length);

var a = new Array(1, 2, true, 4, 5);
trace("// new Array(1, 2, true, 4, 5)");
trace(a);
trace(a.length);

var a = new Array(undefined, false);
trace("// new Array(undefined, false)");
trace(a);
trace(a.length);

var a = ["a"];
trace('// Array.call(a, "b")');
var b = Array.call(a, "b");
trace(a);
trace(b);

var originalArray = Array;
var customPrototype = {};
var hasOwnProperty = Object.prototype.hasOwnProperty;

function describe(value) {
    if (value === originalArray) {
        return "Array";
    }
    if (value === originalArray.prototype) {
        return "Array.prototype";
    }
    if (value === customPrototype) {
        return "customPrototype";
    }
    // Skip `undefined` to not invoke a `_global.Array` getter.
    if (value !== undefined && value === _global.Array) {
        return "_global.Array";
    }
    return typeof value + " " + value;
}

function traceArray(name, array) {
    var ctor = hasOwnProperty.call(array, "constructor") ? "own" : "inherited";
    trace(name + ": __proto__: " + describe(array.__proto__) + ", constructor: " + ctor + " " + describe(array.constructor));
}

function keys(obj) {
    var result = "";
    for (var key in obj) {
        result += key + " ";
    }
    return result;
}

function getArguments() {
    return arguments;
}

var a = [1];
traceArray("[1]", a);
trace("keys: " + keys(a));
trace("own __constructor__: " + hasOwnProperty.call(a, "__constructor__"));
trace("delete constructor: " + delete a.constructor);
a.constructor = 5;
trace("constructor = 5: " + a.constructor);

trace("// _global.Array = function with customPrototype");
_global.Array = function() {
    trace("Array called");
};
_global.Array.prototype = customPrototype;
traceArray("[]", []);
traceArray("arguments", getArguments());
traceArray("Array()", originalArray());
traceArray("'a,b'.split(',')", "a,b".split(","));
traceArray("new Array()", new originalArray());
_global.Array = originalArray;

trace("// _global.Array = {prototype: 5}");
_global.Array = {prototype: 5};
traceArray("[]", []);
_global.Array = originalArray;

trace("// _global.Array = {}");
_global.Array = {};
traceArray("[]", []);
_global.Array = originalArray;

trace("// _global.Array = {} with prototype getter");
_global.Array = {};
_global.Array.addProperty("prototype", function() {
    trace("get Array.prototype");
    return customPrototype;
}, null);
traceArray("[]", []);
_global.Array = originalArray;

trace("// delete _global.Array");
delete _global.Array;
traceArray("[]", []);
_global.Array = originalArray;

trace("// _global.Array getter");
delete _global.Array;
Object.prototype.addProperty.call(_global, "Array", function() {
    trace("get Array");
    return {prototype: customPrototype};
}, null);
traceArray("[]", []);
delete _global.Array;
_global.Array = originalArray;

trace("// local Array variable");
function withLocalArray() {
    var Array = {prototype: customPrototype};
    traceArray("[]", []);
}
withLocalArray();

fscommand("quit");
