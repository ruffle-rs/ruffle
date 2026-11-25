// Loads a movie whose data only arrives after a few frames, and traces
// whenever the state of the target clip changes.
//
// "complete" is how content like Mission in Snowdriftland detects a
// finished load: the clip has the new URL and all of its bytes. Until the
// data of the new movie has arrived, the clip must not look complete.

var c = _root.createEmptyMovieClip("c", 1);
c.loadMovie("child.swf?debug-delay=5");

var frame = 0;
var lastState = "";

_root.onEnterFrame = function() {
    frame++;

    var c = _root.c;
    var complete = c._url.indexOf("child.swf") != -1
        && c.getBytesLoaded() == c.getBytesTotal()
        && c.getBytesTotal() > -1;
    var loaded = c.getBytesTotal() > 0;

    var state = "complete: " + complete + ", loaded: " + loaded;
    if (state != lastState) {
        trace(state);
        lastState = state;
    }

    if (frame == 15) {
        delete _root.onEnterFrame;
    }
};
