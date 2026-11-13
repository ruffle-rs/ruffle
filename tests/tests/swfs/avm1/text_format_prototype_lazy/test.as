// Flash Player doesn't define the properties of TextFormat on its prototype up
// front, but when a TextFormat is constructed, on the prototype that it gets.
// This happens at most once per prototype object, regardless of what it
// contains: existing values are overwritten, and once deleted, they are never
// defined on that prototype again. Calling TextFormat() without `new` doesn't
// count, but TextFormats created by TextField methods do.
//
// This is independent of the native data of the prototype object: native
// objects get them as well, and a prototype that got them can still be made
// native, without getting them again. `makeDate` tries to initialize a Date
// over the object, which only works if it isn't native yet.
var FAKE_DATE_PROTO = {
    __proto__: Date.prototype,
    __constructor__: Date,
    __initializeNative: function() {
        super();
    }
};
function makeDate(o) {
    var original = o.__proto__;
    o.__proto__ = FAKE_DATE_PROTO;
    o.__initializeNative();
    var isDate = (typeof Date.prototype.getDate.call(o)) == "number";
    o.__proto__ = original;
    return isDate;
}
function desc(o, name) {
    return !o.hasOwnProperty(name) ? "-" : typeof o[name];
}
function show(o) {
    var names = ["font", "size", "bold", "align"];
    var s = "";
    for (var i = 0; i < names.length; i++) {
        s += names[i] + "=" + desc(o, names[i]) + " ";
    }
    return s;
}

var p0 = TextFormat.prototype;
trace("startup: " + show(p0));
var a = TextFormat();
trace("after TextFormat() without new: " + show(p0));
var b = new TextFormat("Arial");
trace("after first new: " + show(p0) + "| b.font=" + b.font);
delete p0.font;
var c = new TextFormat("Arial");
trace("deleted font, new again: " + show(p0) + "| c.font=" + c.font);

var p1 = {font: 5};
TextFormat.prototype = p1;
var d = new TextFormat("Arial");
trace("swapped to p1 = {font: 5}, new: " + show(p1) + "| d.__proto__ == p1: " + (d.__proto__ == p1) + ", d.font=" + d.font);
delete p1.font;

TextFormat.prototype = p0;
var e = new TextFormat("Arial");
trace("swapped back to p0, new: " + show(p0) + "| e.font=" + e.font);

TextFormat.prototype = p1;
var f = new TextFormat("Arial");
trace("swapped to p1 again (font deleted), new: " + show(p1) + "| f.font=" + f.font);

var p2 = {};
TextFormat.prototype = p2;
var g = new TextFormat("Arial");
trace("swapped to fresh p2, new: " + show(p2) + "| g.font=" + g.font);
// TextFormats created by TextField methods count too.
createTextField("field", 1, 0, 0, 100, 100);
var p3 = {};
TextFormat.prototype = p3;
var h = field.getTextFormat();
trace("swapped to fresh p3, getTextFormat(): " + show(p3) + "| h.__proto__ == p3: " + (h.__proto__ == p3));
var p4 = {};
TextFormat.prototype = p4;
var i = field.getNewTextFormat();
trace("swapped to fresh p4, getNewTextFormat(): " + show(p4) + "| i.__proto__ == p4: " + (i.__proto__ == p4));

// Plain object, got them, then made a Date.
var n1 = {};
TextFormat.prototype = n1;
new TextFormat();
trace("n1 = {}, new: " + show(n1));
trace("n1, made a Date: " + makeDate(n1));
delete n1.font;
delete n1.size;
new TextFormat();
trace("n1, deleted font and size, new: " + show(n1));
// Plain object, made a Date, then used as prototype.
var n2 = {};
trace("n2 = {}, made a Date: " + makeDate(n2));
TextFormat.prototype = n2;
new TextFormat();
trace("n2, new: " + show(n2));
// Native objects as prototype.
var arr = [1, 2];
TextFormat.prototype = arr;
var j = new TextFormat("Arial");
arr[5] = 0;
trace("[1, 2], new: " + show(arr) + "| length=" + arr.length + ", j.font=" + j.font);
TextFormat.prototype = p0;
var tf = new TextFormat("Foo");
TextFormat.prototype = tf;
var k = new TextFormat("Arial");
trace("a TextFormat, new: " + show(tf) + "| tf.font=" + tf.font + ", k.font=" + k.font);
var dt = new Date(2000, 0, 1);
TextFormat.prototype = dt;
var l = new TextFormat("Arial");
trace("new Date(), new: " + show(dt) + "| year=" + dt.getFullYear() + ", l.font=" + l.font);
TextFormat.prototype = p0;
trace("done");
