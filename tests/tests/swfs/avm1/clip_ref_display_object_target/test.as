// Tests functions and actions that take a display object as their target
// with a reference to a clip whose name now also belongs to another clip.
//
// `ref` is taken from `twin` at depth 50 (at `_x = 100`). Then another clip
// named `twin` is created at depth 40 (at `_x = 300`), so `_level0.twin`
// refers to the new clip. Each case traces which of the two clips it used.

function drawBox(clip) {
    clip.beginFill(0xFF0000);
    clip.moveTo(0, 0);
    clip.lineTo(10, 0);
    clip.lineTo(10, 10);
    clip.lineTo(0, 10);
    clip.endFill();
}

createEmptyMovieClip("probe", 1);
drawBox(probe);

var older = createEmptyMovieClip("twin", 50);
older._x = 100;
drawBox(older);
var ref = twin;

var newer = createEmptyMovieClip("twin", 40);
newer._x = 300;
drawBox(newer);

function show(label) {
    trace(label + ": older._y: " + older._y + ", newer._y: " + newer._y);
}

trace("ref._x: " + ref._x + ", twin._x: " + twin._x);

trace("probe.getBounds(ref).xMin: " + probe.getBounds(ref).xMin);

probe._x = 100;
trace("probe.hitTest(ref) at older: " + probe.hitTest(ref));
probe._x = 300;
trace("probe.hitTest(ref) at newer: " + probe.hitTest(ref));

trace("getProperty(ref, _x): " + getProperty(ref, _x));

setProperty(ref, _y, 5);
show("setProperty(ref, _y, 5)");

with (ref) {
    _y = 6;
}
show("with (ref) { _y = 6; }");

probe.swapDepths(ref);
trace("probe.swapDepths(ref): probe: " + probe.getDepth() + ", older: " + older.getDepth() + ", newer: " + newer.getDepth());
