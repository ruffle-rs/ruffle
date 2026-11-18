// MovieClipLoader events of loads into levels that get unloaded: from the
// onLoadError and onLoadComplete handlers of the load, and in the same frame
// as the load, with and without another load after the unload.

function fileName(url) {
    if (typeof url == "string") {
        return url.substr(url.lastIndexOf("/") + 1);
    }
    return url;
}

function describe(name, level) {
    trace(name + ": " + typeof level + ", " + level + ", " + typeof level._url + ", " + fileName(level._url));
}

function report(mcl, name, unloadOn, event, target, code) {
    trace(name + ": " + event + ", " + target + (code == undefined ? "" : ", " + code));
    if (event == unloadOn) {
        trace(name + ": unloading " + target);
        mcl.unloadClip(target);
    }
}

// Returns a MovieClipLoader that traces its events. If unloadOn is set, the
// handler of that event unloads the target.
function createLoader(name, unloadOn) {
    var mcl = new MovieClipLoader();
    mcl.onLoadStart = function(target) {
        report(mcl, name, unloadOn, "onLoadStart", target);
    };
    mcl.onLoadComplete = function(target) {
        report(mcl, name, unloadOn, "onLoadComplete", target);
    };
    mcl.onLoadInit = function(target) {
        report(mcl, name, unloadOn, "onLoadInit", target);
    };
    mcl.onLoadError = function(target, code) {
        report(mcl, name, unloadOn, "onLoadError", target, code);
    };
    return mcl;
}

var onError = createLoader("_level1 loader", "onLoadError");
var onComplete = createLoader("_level2 loader", "onLoadComplete");
var cancelledA = createLoader("_level3 loader A");
var cancelledB = createLoader("_level3 loader B");
var cancelled = createLoader("_level4 loader");

onError.loadClip("missing.swf", 1);
onComplete.loadClip("first.swf", 2);
cancelledA.loadClip("first.swf", 3);
cancelledA.unloadClip(3);
cancelledB.loadClip("second.swf", 3);
cancelled.loadClip("first.swf", 4);
cancelled.unloadClip(4);
trace("// Loads started");

setTimeout(function() {
    trace("// After the loads");
    describe("_level1", _level1);
    describe("_level2", _level2);
    describe("_level3", _level3);
    describe("_level4", _level4);
}, 1000);
