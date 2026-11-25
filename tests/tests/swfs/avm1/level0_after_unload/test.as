// _level0 unloaded with unloadMovieNum, observed from a movie on _level1.

this.onUnload = function() {
    trace("onUnload: " + this);
};

function childLoaded() {
    trace("// Unloading _level0");
    unloadMovieNum(0);
}

loadMovieNum("sub/child.swf", 1);
