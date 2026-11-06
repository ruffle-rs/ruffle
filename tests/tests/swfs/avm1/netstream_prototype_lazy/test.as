// Flash Player doesn't define most properties of NetStream on its prototype
// up front, but when a NetStream is created with a connected NetConnection,
// on the prototype it gets, at most once per prototype object. Unlike for
// other classes, this also happens without `new`.
//
// The properties are listed with for-in on a fresh `{}` prototype, in the
// order Flash Player enumerates them.
//
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

var nc = new NetConnection();
nc.connect(null);
var closedNc = new NetConnection();
closedNc.connect(null);
closedNc.close();
trace("startup, NetStream.prototype.time: " + NetStream.prototype.hasOwnProperty("time"));
check("new NetStream(nc)", NetStream, function() { return new NetStream(nc); }, true);
check("new NetStream()", NetStream, function() { return new NetStream(); });
check("new NetStream(unconnected)", NetStream, function() { return new NetStream(new NetConnection()); });
check("new NetStream(closed)", NetStream, function() { return new NetStream(closedNc); });
check("NetStream(nc) without new", NetStream, function(p) { return callWithout(NetStream, p, nc); });
checkSuper("NetStream", NetStream, nc);
checkOnce(NetStream, "time", function() { new NetStream(nc); });
trace("done");
