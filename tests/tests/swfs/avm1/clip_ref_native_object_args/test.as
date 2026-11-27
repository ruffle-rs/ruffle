// Tests functions that read properties of an object argument with a
// reference to a clip whose name now also belongs to another clip.
//
// For each case, `ref` is taken from a clip named `name`, and then another
// clip with the same name is created at a lower depth, so that `name`
// refers to the new clip. The two clips have different property values,
// and each case traces which of them the function used.

function setup(name, depth) {
    var older = createEmptyMovieClip(name, depth);
    var ref = _root[name];
    var newer = createEmptyMovieClip(name, depth - 1);
    older.tag = "older";
    newer.tag = "newer";
    return {ref: ref, older: older, newer: newer};
}

function first() {
    return arguments[0];
}

function getTag() {
    return this.tag;
}

createEmptyMovieClip("container", 1);
container._x = 100;

var clips = setup("point", 11);
var ref = clips.ref;
var older = clips.older;
var newer = clips.newer;
older.x = 1;
older.y = 1;
newer.x = 2;
newer.y = 2;
container.localToGlobal(ref);
trace("localToGlobal: older.x: " + older.x + ", newer.x: " + newer.x);

var clips = setup("args", 21);
var ref = clips.ref;
clips.older.length = 1;
clips.older[0] = "older";
clips.newer.length = 1;
clips.newer[0] = "newer";
trace("Function.apply: " + first.apply(null, ref));

var clips = setup("matrix", 31);
var ref = clips.ref;
clips.older.length = 1;
clips.older[0] = 1;
clips.newer.length = 1;
clips.newer[0] = 2;
var filter = new flash.filters.ColorMatrixFilter(ref);
trace("ColorMatrixFilter: " + filter.matrix[0]);

var clips = setup("style", 41);
var ref = clips.ref;
clips.older.color = "#000001";
clips.newer.color = "#000002";
var styleSheet = new TextField.StyleSheet();
trace("StyleSheet.transform: " + styleSheet.transform(ref).color);

var clips = setup("init", 51);
var ref = clips.ref;
attachMovie("symbol", "attached", 100, ref);
trace("attachMovie: " + attached.tag);

var clips = setup("thisArg", 56);
var ref = clips.ref;
trace("Function.call: " + getTag.call(ref));

var clips = setup("timer", 61);
var ref = clips.ref;
var interval;

function tick() {
    trace("setInterval: " + this.tag);
    clearInterval(interval);
}

clips.older.tick = tick;
clips.newer.tick = tick;
interval = setInterval(ref, "tick", 1);

var frames = 0;
onEnterFrame = function() {
    frames++;

    if (frames == 2) {
        trace("Done");
        delete onEnterFrame;
    }
};
