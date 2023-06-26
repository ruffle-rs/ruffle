// SWF Version 7

trace("Child (7) loaded");

var mc = createEmptyMovieClip("clip", getNextHighestDepth());
var frameCount = 0;
this.onEnterFrame = function() {
	frameCount++;
	if (frameCount == 3) {
		trace("Child (7) " + this.getBounds(mc).xMin);
		this.onEnterFrame = null;
	}
};
