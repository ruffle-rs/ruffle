// Tests LoadVars.sendAndLoad with a movie clip as the target (inline and in
// a variable) and with a LoadVars object as the target. It traces the return
// values, the onData events of the clip and the onLoad events of the
// LoadVars target, and after the loads had time to finish, the loaded
// variable `b` of each target.

createEmptyMovieClip("target", getNextHighestDepth());
target.onData = function(src) {
    trace("target.onData: " + src);
};

var loadVarsTarget = new LoadVars();
loadVarsTarget.onLoad = function(success) {
    trace("loadVarsTarget.onLoad: " + success);
};

var lv = new LoadVars();
lv.a = "1";

trace("clip, GET: " + lv.sendAndLoad("data.txt", _root.target, "GET"));
trace("clip, POST: " + lv.sendAndLoad("data.txt", _root.target, "POST"));

var clipRef = _root.target;
trace("clip in a variable: " + lv.sendAndLoad("data.txt", clipRef, "GET"));
trace("LoadVars: " + lv.sendAndLoad("data.txt", loadVarsTarget, "GET"));

var frames = 0;
onEnterFrame = function() {
    frames++;

    if (frames == 3) {
        trace("target.b: " + target.b);
        trace("loadVarsTarget.b: " + loadVarsTarget.b);
        trace("Done");
        delete onEnterFrame;
    }
};
