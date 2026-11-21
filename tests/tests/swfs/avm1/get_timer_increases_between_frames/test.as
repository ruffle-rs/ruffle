// Tests that getTimer() increases between frames, by tracing for each of 30
// frames whether it is greater than in the previous one.

var frames = 0;
var last = -1;

_root.onEnterFrame = function() {
    var time = getTimer();
    frames++;
    trace("frame " + frames + ": increased " + (time > last));
    last = time;

    if (frames == 30) {
        delete _root.onEnterFrame;
        trace("Done");
    }
};
