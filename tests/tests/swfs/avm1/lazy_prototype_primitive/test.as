// The native constructors that define lazily defined properties on the
// prototype of the new object don't define them anywhere if that prototype
// isn't an object: not on the object, the original prototype of the class,
// or the prototypes of the primitive types. With a `{}` prototype, they're
// defined on it.
var nc = new NetConnection();
nc.connect(null);
var values = [5, "s", true, null, undefined];
var names = ["5", "\"s\"", "true", "null", "undefined"];
var builtins = [["Object", Object.prototype], ["Number", Number.prototype], ["String", String.prototype], ["Boolean", Boolean.prototype]];
// `hasOwnProperty` can't be looked up on an object with a primitive __proto__.
function has(o, k) {
    return Object.prototype.hasOwnProperty.call(o, k);
}
// Lists where `prop` was defined, deleting it from the builtin prototypes.
function where(o, orig, prop) {
    var r = [];
    if (has(o, prop)) {
        r.push("the object");
    }
    var p = o.__proto__;
    if (typeof p == "object" && p != null && has(p, prop)) {
        r.push("its prototype");
    }
    if (has(orig, prop)) {
        r.push("the original prototype");
    }
    for (var i = 0; i < builtins.length; i++) {
        if (has(builtins[i][1], prop)) {
            r.push(builtins[i][0] + ".prototype");
            delete builtins[i][1][prop];
        }
    }
    var t = typeof p;
    return "__proto__ is " + (t == "object" && p != null ? "an object" : t + " " + p) + ", " + prop + " on " + (r.length ? r.join(", ") : "nothing");
}
// Sets `C.prototype` to each value and to a `{}`, and creates an instance.
function check(name, C, prop) {
    var orig = C.prototype;
    for (var i = 0; i <= values.length; i++) {
        C.prototype = (i < values.length) ? values[i] : {};
        var o = new C(nc);
        C.prototype = orig;
        trace(name + ".prototype = " + (i < values.length ? names[i] : "{}") + ": " + where(o, orig, prop));
    }
}

check("TextFormat", TextFormat, "font");
check("TextField", TextField, "scroll");
check("Button", Button, "tabIndex");
check("XML", XML, "status");
check("NetStream", NetStream, "time");
check("PrintJob", PrintJob, "paperWidth");
// NetStream also does this without `new`.
for (var i = 0; i <= values.length; i++) {
    var o = {};
    o.__proto__ = (i < values.length) ? values[i] : {};
    o.f = NetStream;
    o.f(nc);
    trace("NetStream() without new, __proto__ = " + (i < values.length ? names[i] : "{}") + ": " + where(o, NetStream.prototype, "time"));
}
trace("done");
