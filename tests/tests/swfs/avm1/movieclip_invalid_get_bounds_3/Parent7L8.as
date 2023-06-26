// SWF Version 7

var mc = createEmptyMovieClip("clip", getNextHighestDepth());
trace("Parent (7) " + mc.getBounds(this).xMin);

var loader = new MovieClipLoader();
loader.addListener(this);
loader.loadClip("Child8.swf", mc);

function onLoadInit(mc:MovieClip) {
	trace("Parent (7) " + mc.getBounds(this).xMin);

	var frameCount = 0;
	this.onEnterFrame = function() {
		frameCount++;
		if (frameCount == 5) {
			trace("Parent (7) " + mc.getBounds(this).xMin);
			this.onEnterFrame = null;
		}
	}
}
