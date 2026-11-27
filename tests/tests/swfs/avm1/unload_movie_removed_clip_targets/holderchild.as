this.createEmptyMovieClip("marker", 100);
var removed = this.createEmptyMovieClip("removed", 1);
var ref = removed;
removed.removeMovieClip();
trace("holderchild.swf: unloadMovie(reference to a removed clip)");
unloadMovie(ref);
