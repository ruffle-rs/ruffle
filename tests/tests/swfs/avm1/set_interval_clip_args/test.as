// Tests setInterval with a movie clip as the object whose method is called,
// and with a stale clip reference as an extra argument for the callback.
//
// The stale reference is taken from a clip that is then removed and created
// again. After setInterval, that clip is renamed to `stale_moved` (with
// `_x = 3`) and another clip is created with the old name (with `_x = 9`).
// The callbacks trace which clip they got.

createEmptyMovieClip("owner", getNextHighestDepth());
owner.tick = function(label) {
    trace("owner.tick: this = " + this + ", label = " + label);
    clearInterval(ownerInterval);
};

var ownerInterval = setInterval(owner, "tick", 1, "clip");

createEmptyMovieClip("stale", 100);
var staleRef = stale;
stale.removeMovieClip();
createEmptyMovieClip("stale", 100);

function callback(ref) {
    trace("callback: ref = " + ref + ", _x = " + ref._x);
    clearInterval(callbackInterval);
}

var callbackInterval = setInterval(callback, 1, staleRef);

stale._name = "stale_moved";
stale_moved._x = 3;
createEmptyMovieClip("stale", 101);
stale._x = 9;

var frames = 0;
onEnterFrame = function() {
    frames++;

    if (frames == 4) {
        trace("Done");
        delete onEnterFrame;
    }
};
