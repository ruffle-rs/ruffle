// SWF Version 7

var mc = createEmptyMovieClip("clip", getNextHighestDepth());
trace("Parent (7) " + mc.getBounds(this).xMin);

var loader = new MovieClipLoader();
loader.addListener(this);
loader.loadClip("Child8TestingThis.swf", mc);

function onLoadInit(mc:MovieClip) {
	trace("Parent (7) " + mc.getBounds(this).xMin);
}
