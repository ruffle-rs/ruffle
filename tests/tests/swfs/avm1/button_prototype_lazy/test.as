// Flash Player doesn't define `tabIndex` on Button.prototype up front, but
// when a Button is created, on the prototype it gets, at most once per
// prototype object. This is done by the Button constructor, which is also
// called for Button display objects, so it also happens for `new Button()`,
// but not without `new`.
//
// The properties are listed with for-in on a fresh `{}` prototype, in the
// order Flash Player enumerates them.
//
// test.swf (from test.swfasm) places a Button as `b` on frame 2. Going to
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
        trace("startup, Button.prototype.tabIndex: " + Button.prototype.hasOwnProperty("tabIndex"));
        check("new Button()", Button, function() { return new Button(); }, true);
        check("placed", Button, function() { return place("b"); });
        check("Button() without new", Button, function(p) { return callWithout(Button, p); });
        checkSuper("Button", Button);
        checkOnce(Button, "tabIndex", function() { new Button(); });
        trace("done");
    };
}
