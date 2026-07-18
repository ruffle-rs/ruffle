MovieClip.prototype.onConstruct = function() {
    trace("onConstruct, throwing");
    throw "some error";
};

try {
    createEmptyMovieClip("clip_d", 8);
    throw "other error";
} catch (err) {
    trace("Caught: " + err);
}

trace("clip_d.depth = " + clip_d.getDepth());

MovieClip.prototype.onConstruct = function() {
    trace("onConstruct");
    var mc = this;
    trace("  name: " + mc._name);
    trace("  depth: " + mc.getDepth());
};

createEmptyMovieClip("clip_a", 4);
createEmptyMovieClip("clip_b", 5);
createEmptyMovieClip("clip_b", 6);

clip_a.createEmptyMovieClip("clip_c", 17);

createEmptyMovieClip("child", 19);

new MovieClipLoader().loadClip("child.swf", child);
