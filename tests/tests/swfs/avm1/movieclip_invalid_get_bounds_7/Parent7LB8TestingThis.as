// SWF Version 7

var mc = createEmptyMovieClip("clip", getNextHighestDepth());
var loader = new MovieClipLoader();
loader.addListener(this);
loader.loadClip("BoundsChild8TestingThis.swf", mc);

function onLoadInit(mc:MovieClip) {
	unloadMovie(mc);

	var frameCount = 0;
	this.onEnterFrame = function() {
		frameCount++;
		if (frameCount == 3) {
			trace("Parent (7) " + mc.getBounds(this).xMin);
			this.onEnterFrame = null;
		}
	}
}
