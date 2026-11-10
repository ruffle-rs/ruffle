// A root movie larger than Flash Player's 4 KiB read chunk: frame 1 runs
// before the rest of the file has arrived, which comes right after it.
// test.swf is this script on frame 1 plus filler frames (DefineBinaryData),
// uncompressed: frames 2 and 3 are in the first chunk, frames 4 to 6 aren't.
function log(s) {
    _root.tf.text += s + "\n";
    trace(s);
}
function state(what) {
    log(what + ": _currentframe " + _root._currentframe + ", _framesloaded " + _root._framesloaded + "/" + _root._totalframes + ", bytes " + _root.getBytesLoaded() + "/" + _root.getBytesTotal());
}
if (_root.started != true) {
    _root.started = true;
    _root.createTextField("tf", 1, 0, 0, 550, 400);
    _root.tf.multiline = true;
    state("frame 1");
    ifFrameLoaded (2) {
        log("frame 1: ifFrameLoaded(2)");
    }
    ifFrameLoaded (3) {
        log("frame 1: ifFrameLoaded(3)");
    }
    ifFrameLoaded (4) {
        log("frame 1: ifFrameLoaded(4)");
    }
    _root.ticks = 0;
    _root.onEnterFrame = function() {
        _root.ticks = _root.ticks + 1;
        state("enterFrame " + _root.ticks);
        ifFrameLoaded (6) {
            log("enterFrame " + _root.ticks + ": ifFrameLoaded(6)");
        }
        if (_root.ticks == 3) {
            delete _root.onEnterFrame;
        }
    };
}
