class Test {
    static function main() {
        var target:MovieClip = _root.createEmptyMovieClip("target", 1);
        target.beginFill(0x33CC66, 100);
        target.moveTo(0, 0);
        target.lineTo(60, 0);
        target.lineTo(60, 60);
        target.lineTo(0, 60);
        target.lineTo(0, 0);
        target.endFill();
        target._x = 100; target._y = 100;

        // Make sure it behaves like a button for drag events.
        target.trackAsMenu = true;
        target.useHandCursor = false;

        target.onDragOver = function() {
            trace("OK");
            fscommand("quit");
        };
    }
}
