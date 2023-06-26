// SWF Version 8

trace("8LTrace8 loaded");

var loader = new MovieClipLoader();
loader.addListener(this);
var mc = createEmptyMovieClip("testMovieClip", getNextHighestDepth());
loader.loadClip("Trace8.swf", mc);

function onLoadInit(_) {
	trace("8LTrace8 finished");
}
