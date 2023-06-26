// SWF Version 8

trace("Child (8) loaded");

var mc = createEmptyMovieClip("clip", getNextHighestDepth());

var frameCount = 0;
this.onEnterFrame = function() {
	frameCount++;
	if (frameCount == 3) {
		trace("Child (8) " + this.getRect(mc).xMin);
		this.onEnterFrame = null;
	}
};
