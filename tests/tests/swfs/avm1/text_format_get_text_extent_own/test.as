// Flash Player doesn't define `getTextExtent` on TextFormat.prototype, but
// the TextFormat constructor defines it on each TextFormat, as an ordinary
// (enumerable and deletable) property, with a new function object each time.
// This also happens for `super()`, and for TextFormats created by TextField
// methods, but not without `new`. It happens at most once per object: not if
// the object already has native data, of any class.
function where(label, o) {
    var enumerated = false;
    for (var k in o) {
        if (k == "getTextExtent") {
            enumerated = true;
        }
    }
    trace(label + ": own=" + o.hasOwnProperty("getTextExtent") + ", typeof=" + typeof o.getTextExtent + ", enumerable=" + enumerated);
}
where("TextFormat.prototype", TextFormat.prototype);
var a = new TextFormat();
var b = new TextFormat();
where("new TextFormat()", a);
where("TextFormat.prototype, after new TextFormat()", TextFormat.prototype);
trace("same function on two instances: " + (a.getTextExtent === b.getTextExtent));
trace("works: " + typeof a.getTextExtent("hello").width);
var c = {};
c.f = TextFormat;
c.f();
where("TextFormat() without new", c);
var t = _root.createTextField("t", 1, 0, 0, 100, 20);
where("getTextFormat()", t.getTextFormat());
where("getNewTextFormat()", t.getNewTextFormat());
var Sub = function() {
    super();
};
Sub.prototype.__proto__ = TextFormat.prototype;
Sub.prototype.__constructor__ = TextFormat;
ASSetPropFlags(Sub.prototype, "__constructor__", 1);
where("super() in a subclass", new Sub());
a.getTextExtent = 5;
trace("assigned: " + a.getTextExtent);
var deleted = delete a.getTextExtent;
trace("deleted: " + deleted + ", typeof after: " + typeof a.getTextExtent);

// Running the constructor again, with super().
var Again = function() {
    trace("before super(): " + typeof this.getTextExtent);
    super();
    trace("after super(): " + typeof this.getTextExtent);
    var deleted = delete this.getTextExtent;
    super();
    trace("deleted, after super() again: " + typeof this.getTextExtent);
    this.getTextExtent = 5;
    super();
    trace("assigned 5, after super() again: " + typeof this.getTextExtent);
};
Again.prototype.__proto__ = TextFormat.prototype;
Again.prototype.__constructor__ = TextFormat;
ASSetPropFlags(Again.prototype, "__constructor__", 1);
new Again();

// Running the constructor on objects with native data of other classes.
var FAKE_TEXT_FORMAT_PROTO = {
    __proto__: TextFormat.prototype,
    __constructor__: TextFormat,
    __initializeNative: function() {
        super();
    }
};
ASSetPropFlags(FAKE_TEXT_FORMAT_PROTO, "__constructor__", 1);
function makeTextFormat(o) {
    var original = o.__proto__;
    o.__proto__ = FAKE_TEXT_FORMAT_PROTO;
    o.__initializeNative();
    o.__proto__ = original;
    return o;
}
where("TextFormat constructor on a plain object", makeTextFormat({}));
where("TextFormat constructor on a Date", makeTextFormat(new Date(2000, 0, 1)));
where("TextFormat constructor on an Array", makeTextFormat([1, 2]));
trace("done");
