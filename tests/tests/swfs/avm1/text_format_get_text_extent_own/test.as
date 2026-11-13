// Flash Player doesn't define `getTextExtent` on TextFormat.prototype, but
// the TextFormat constructor defines it on each TextFormat, as an ordinary
// (enumerable and deletable) property, with a new function object each time.
// This also happens for `super()`, and for TextFormats created by TextField
// methods, but not without `new`.
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
trace("done");
