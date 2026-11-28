stop();

var mc = createEmptyMovieClip("mc", 1);
var frame = 0;

mc.getSWFVersion = function() {
    return "modified";
};

this.onEnterFrame = function() {
    frame++;

    if (frame == 3) {
        trace("before=" + mc.getSWFVersion());
        mc.unloadMovie();
        trace("immediate=" + mc.getSWFVersion());
    }

    if (frame == 8) {
        trace("later=" + mc.getSWFVersion());
        trace("METHOD_OVERRIDE_COMPLETE");
        delete this.onEnterFrame;
    }
};
