var mcl = new MovieClipLoader();
mcl.onLoadComplete = function() {
    trace("onLoadComplete");
};
mcl.onLoadInit = function() {
    trace("onLoadInit");
};
mcl.onLoadError = function(t, e, s) {
    trace("onLoadError: " + t + " " + e + " " + s);
};
createEmptyMovieClip("child",1);
mcl.loadClip("child.swf",child);
