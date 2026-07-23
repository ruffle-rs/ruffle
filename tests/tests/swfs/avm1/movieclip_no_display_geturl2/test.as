class Test {
    static function main() {
        var mc = _root.createEmptyMovieClip("mc", 1);
        mc.removeMovieClip();
        // This should NOT unload _root.
        loadMovie("", mc);

        // Check on the next frame that _root is still alive.
        _root.onEnterFrame = function() {
            delete _root.onEnterFrame;
            trace("Still alive");
        };
    }
}
