// Tests that getTimer() never goes backwards, even right after a frame that
// called it in a busy loop, and when it's called from an interval between
// frames.

var max = 0;
var decreased = false;

function check(time) {
    if (time < max) {
        decreased = true;
    }

    if (time > max) {
        max = time;
    }
}

var interval = setInterval(function() {
    check(getTimer());
}, 1);

var frames = 0;
_root.onEnterFrame = function() {
    for (var i = 0; i < 500; i++) {
        check(getTimer());
    }

    frames++;

    if (frames == 5) {
        delete _root.onEnterFrame;
        clearInterval(interval);
        trace("getTimer() never decreased: " + !decreased);
    }
};
