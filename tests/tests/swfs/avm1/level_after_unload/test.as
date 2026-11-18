// Levels unloaded with unloadMovieNum, loadMovieNum with an empty URL and
// the unloadMovie method, and unloadMovieNum on a level that doesn't exist.
// Each level has an onUnload handler, a clip with one, an onEnterFrame
// handler, an interval and a function.

var loadedCount = 0;
var saved = [];
var functions = [];
var enterFrames = {};
var intervals = {};
var lastEnterFrames = {};
var lastIntervals = {};

function fileName(url) {
    if (typeof url == "string") {
        return url.substr(url.lastIndexOf("/") + 1);
    }
    return url;
}

function describe(name, level) {
    trace(name + ": " + typeof level + ", " + level + ", " + typeof level._url + ", " + fileName(level._url));
}

function describeAll() {
    describe("_level1 (unloadMovieNum)", _level1);
    describe("_level2 (loadMovieNum with an empty URL)", _level2);
    describe("_level3 (unloadMovie method)", _level3);
    describe("_level4 (unloadMovieNum, never loaded)", _level4);
    for (var i = 1; i <= 3; i++) {
        describe("reference to _level" + i, saved[i]);
        describe("reference to _level" + i + ".inner", saved[i].inner);
    }
}

function state(n) {
    return (n > 0 ? "running" : "stopped");
}

// Traces whether the onEnterFrame handlers and intervals of the levels ran
// since the last call.
function checkRunning(silent) {
    for (var i = 1; i <= 3; i++) {
        var name = "_level" + i;
        if (!silent) {
            trace(name + ": onEnterFrame " + state(enterFrames[name] - lastEnterFrames[name]) + ", interval " + state(intervals[name] - lastIntervals[name]));
        }
        lastEnterFrames[name] = enterFrames[name];
        lastIntervals[name] = intervals[name];
    }
}

function callFunctions() {
    for (var i = 1; i <= 3; i++) {
        trace("_level" + i + " function: " + functions[i]());
    }
}

// Called by each child when its frame runs.
function childLoaded() {
    loadedCount++;
    if (loadedCount == 3) {
        saved[1] = _level1;
        saved[2] = _level2;
        saved[3] = _level3;
        functions[1] = _level1.describeSelf;
        functions[2] = _level2.describeSelf;
        functions[3] = _level3.describeSelf;
        checkRunning(true);
        setTimeout(unload, 300);
    } else if (loadedCount > 3) {
        trace("// Child loaded into " + _level1);
    }
}

function unload() {
    trace("// Before unloading");
    describeAll();
    checkRunning();
    callFunctions();
    unloadMovieNum(1);
    loadMovieNum("", 2);
    _level3.unloadMovie();
    unloadMovieNum(4);
    trace("// Right after unloading");
    describeAll();
    setTimeout(afterUnload, 500);
}

function afterUnload() {
    trace("// After unloading");
    describeAll();
    for (var i = 1; i <= 3; i++) {
        var name = "_level" + i;
        trace(name + ": onEnterFrame called " + (enterFrames[name] - lastEnterFrames[name]) + " times");
    }
    checkRunning();
    setTimeout(later, 500);
}

function later() {
    trace("// Later");
    checkRunning();
    callFunctions();
    trace("// Loading a movie into _level1 again");
    loadMovieNum("sub/child.swf", 1);
    setTimeout(afterReload, 500);
}

function afterReload() {
    trace("// After loading again");
    describe("_level1", _level1);
    describe("reference to _level1", saved[1]);
    checkRunning();
}

loadMovieNum("sub/child.swf", 1);
loadMovieNum("sub/child.swf", 2);
loadMovieNum("sub/child.swf", 3);
