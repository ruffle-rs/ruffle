function checkLoadComplete() {
    // In FP, MovieClipLoader's normal methods for checking
    // if an MC has finished loading don't work under AVM2,
    // so we need to do it this way instead.
    if (clip.getBytesLoaded() == clip.getBytesTotal() && clip.getBytesTotal() > 4) {
        clearInterval(interval);
        clip.onRelease = function() {
            trace("avm1 child clicked");
        }
    }
}

var clip = this.createEmptyMovieClip("clip", 1);
var mcl = new MovieClipLoader();
var interval = setInterval(checkLoadComplete, 4);

mcl.loadClip("avm1_child.swf", clip);
