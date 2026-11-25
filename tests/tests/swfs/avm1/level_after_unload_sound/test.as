// Sounds played by levels, after the levels are unloaded: a sound targeting
// the level, one without a target, and a streamed one. The levels play their
// sounds one after another, each in a phase of 10 frames.
// The current frame and phase are shown on the stage, so that the frames with
// audio can be checked in Flash Player. onEnterFrame first runs in frame 2, so
// the frame of the movie is one more than the number of onEnterFrame calls.

var labels = [
    "_level1 (targeting the level)",
    "_level2 (without a target)",
    "_level3 (streamed)"
];
var sound;
var lastPosition;
var frame = 0;
var phase = "";

createTextField("frameText", 1, 10, 10, 500, 100);
var format = new TextFormat();
format.size = 32;
frameText.setNewTextFormat(format);
frameText.text = "frame 1";

// Traces whether the position of the sound advanced since the last call.
function checkPosition(level, silent) {
    var position = sound.position;
    if (!silent) {
        trace(labels[level - 1] + ": " + (position > lastPosition ? "advancing" : "not advancing"));
    }
    lastPosition = position;
}

// Called by each child when its frame runs.
function childLoaded(name, childSound) {
    sound = childSound;
    phase = name + " playing";
}

onEnterFrame = function() {
    frame++;
    var level = Math.floor((frame - 1) / 10) + 1;
    var step = (frame - 1) % 10;
    if (level > 3) {
        if (level == 4 && step == 0) {
            phase = "end";
            trace("// End");
        }
    } else if (step == 0) {
        phase = labels[level - 1] + " loading";
        loadMovieNum("child.swf", level);
    } else if (step == 2) {
        checkPosition(level, true);
    } else if (step == 3) {
        trace("// Before unloading");
        checkPosition(level, false);
        phase = labels[level - 1] + " unloaded";
        unloadMovieNum(level);
    } else if (step == 4) {
        checkPosition(level, true);
    } else if (step == 5) {
        trace("// After unloading");
        checkPosition(level, false);
    }
    frameText.text = "frame " + (frame + 1) + "\n" + phase;
};
