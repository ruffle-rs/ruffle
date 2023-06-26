// SWF Version 8

var mc = createEmptyMovieClip("clip", getNextHighestDepth());
trace("SP8 loaded");

var loader = new MovieClipLoader();
loader.addListener(this);
loader.loadClip("Child7.swf", mc);

function onLoadInit(mc:MovieClip) {
	trace("SP8 running");

	var frameCount = 0;
	this.onEnterFrame = function() {
		frameCount++;
		if (frameCount == 5) {
			trace("SP8 finished");
			this.onEnterFrame = null;
		}
	}
}
