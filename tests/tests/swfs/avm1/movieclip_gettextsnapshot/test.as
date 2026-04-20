var oldTextSnapshot = _global.TextSnapshot;
_global.TextSnapshot = function(a, b, c) {
    trace("TextSnapshot instantiated:");
    trace("  a=" + a);
    trace("  b=" + b);
    trace("  c=" + c);
    trace("  typeof a=" + typeof a);
    trace("  typeof b=" + typeof b);
    trace("  typeof c=" + typeof c);
    return "return value";
};

function printValue(name, value) {
    trace(name + "=" + value + " (" + (typeof value) + ")");
    return value;
}

function printResult(ts) {
    trace("is _global.TextSnapshot: " + (ts.constructor === _global.TextSnapshot));
    trace("is oldTextSnapshot: " + (ts.constructor === oldTextSnapshot));
}

trace("/////// Within " + this._target);

printValue("this.getTextSnapshot", this.getTextSnapshot);
printValue("_root.getTextSnapshot", _root.getTextSnapshot);
printValue("_root.child.getTextSnapshot", _root.child.getTextSnapshot);

var ts = printValue("this.getTextSnapshot()", this.getTextSnapshot());
printResult(ts);

var ts = printValue("_root.getTextSnapshot()", _root.getTextSnapshot());
printResult(ts);

var ts = printValue("_root.child.getTextSnapshot()", _root.child.getTextSnapshot());
printResult(ts);

createEmptyMovieClip("clip1", 10);
printValue("clip1.getTextSnapshot", clip1.getTextSnapshot);
var ts = printValue("clip1.getTextSnapshot()", clip1.getTextSnapshot());
printResult(ts);

var clip2 = _root.createEmptyMovieClip("clip2", 10);
printValue("clip2.getTextSnapshot", clip2.getTextSnapshot);
var ts = printValue("clip2.getTextSnapshot()", clip2.getTextSnapshot());
printResult(ts);
