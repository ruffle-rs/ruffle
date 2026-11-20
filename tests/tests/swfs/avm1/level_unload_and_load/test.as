// Levels unloaded and loaded again in the same frame, levels unloaded while
// a movie is loading into them, and loadMovieNum with an empty URL on a level
// that doesn't exist.

var firstCount = 0;
var mcl = new MovieClipLoader();

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
    describe("_level1 (unloadMovieNum, then loadMovieNum)", _level1);
    describe("_level2 (unloadClip, then loadClip)", _level2);
    describe("_level3 (loadMovieNum, then unloadMovieNum)", _level3);
    describe("_level5 (loadMovieNum with an empty URL, never loaded)", _level5);
    describe("_level6 (loadMovieNum with an empty URL in a variable, never loaded)", _level6);
}

// Called by each movie when its frame runs.
function movieLoaded(name, level) {
    if (name == "first.swf") {
        firstCount++;
        if (firstCount == 2) {
            setTimeout(start, 300);
        }
        return;
    }
    trace("// " + name + " loaded into " + level);
}

function start() {
    trace("// Before");
    describeAll();
    unloadMovieNum(1);
    loadMovieNum("second.swf", 1);
    mcl.unloadClip(2);
    mcl.loadClip("second.swf", 2);
    loadMovieNum("second.swf", 3);
    unloadMovieNum(3);
    loadMovieNum("", 5);
    var emptyUrl = "";
    var level = 6;
    loadMovieNum(emptyUrl, level);
    trace("// Right after");
    describeAll();
    setTimeout(after, 500);
}

function after() {
    trace("// After");
    describeAll();
}

loadMovieNum("first.swf", 1);
loadMovieNum("first.swf", 2);
