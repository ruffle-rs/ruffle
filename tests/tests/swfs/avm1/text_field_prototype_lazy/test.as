// Flash Player doesn't define most properties of TextField on its prototype
// up front, but when a TextField is created, on the prototype it gets, at
// most once per prototype object. This is done by the TextField constructor,
// which is also called for TextField display objects, so it also happens for
// `new TextField()`, but not without `new`, or with a replaced TextField class.
//
// The properties are listed with for-in on a fresh `{}` prototype, in the
// order Flash Player enumerates them.
//
// test.swf (from test.swfasm) places a TextField as `t` on frame 2. Going to
// frame 1 and back to frame 2 creates a new one.
function own(o) {
    var r = [];
    for (var k in o) {
        if (o.hasOwnProperty(k)) {
            r.push(k);
        }
    }
    return r.join(",");
}
function count(o) {
    var n = 0;
    for (var k in o) {
        if (o.hasOwnProperty(k)) {
            n++;
        }
    }
    return n;
}
// Calls `make` with a fresh prototype for `C`, and lists what it got.
function check(label, C, make, full) {
    var orig = C.prototype;
    var fresh = {};
    C.prototype = fresh;
    var o = make(fresh);
    C.prototype = orig;
    var proto = (o.__proto__ == fresh) ? "" : " (__proto__ is not the fresh prototype)";
    trace(label + ": " + (full ? own(fresh) : count(fresh)) + proto);
}
// Calls `C` as a method of an object with the prototype `proto`, without `new`.
function callWithout(C, proto, a0, a1) {
    var o = {};
    o.__proto__ = proto;
    o.f = C;
    o.f(a0, a1);
    return o;
}
// A subclass of `C`, whose constructor calls super(a0, a1).
function checkSuper(label, C, a0, a1) {
    var orig = C.prototype;
    var cp = {};
    C.prototype = cp;
    var sp = {};
    sp.__proto__ = cp;
    sp.__constructor__ = C;
    ASSetPropFlags(sp, "__constructor__", 1);
    var Sub = function() {
        super(a0, a1);
    };
    Sub.prototype = sp;
    new Sub();
    C.prototype = orig;
    trace("super() in a subclass: " + count(sp) + " on its prototype, " + count(cp) + " on " + label + ".prototype");
}
// Creates an instance twice with the same prototype, deleting `name` in between.
function checkOnce(C, name, make) {
    var orig = C.prototype;
    var p = {};
    C.prototype = p;
    make();
    var before = p.hasOwnProperty(name);
    delete p[name];
    make();
    C.prototype = orig;
    trace(name + " deleted, created again: " + before + " -> " + p.hasOwnProperty(name));
}
function place(name) {
    _root.gotoAndStop(1);
    _root.gotoAndStop(2);
    return _root[name];
}

if (this.started == undefined) {
    this.started = true;
    stop();
    this.onEnterFrame = function() {
        delete this.onEnterFrame;
        trace("startup, TextField.prototype.text: " + TextField.prototype.hasOwnProperty("text"));
        check("new TextField()", TextField, function() { return new TextField(); }, true);
        check("createTextField", TextField, function() { return _root.createTextField("t1", 10, 0, 0, 10, 10); });
        check("placed", TextField, function() { return place("t"); });
        check("TextField() without new", TextField, function(p) { return callWithout(TextField, p); });
        var origTF = _global.TextField;
        var replaced = {};
        _global.TextField = function() {};
        _global.TextField.prototype = replaced;
        var t2 = _root.createTextField("t2", 11, 0, 0, 10, 10);
        _global.TextField = origTF;
        trace("createTextField with a replaced TextField class: " + count(replaced) + ((t2.__proto__ == replaced) ? "" : " (__proto__ is not its prototype)"));
        checkSuper("TextField", TextField);
        checkOnce(TextField, "text", function() { new TextField(); });
        trace("done");
    };
}
