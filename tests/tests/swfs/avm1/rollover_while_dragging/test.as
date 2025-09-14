class Test {
    static function main() {
        var dragger:MovieClip = _root.createEmptyMovieClip("dragger", 1);
        dragger.beginFill(0xCC3333, 100);
        dragger.moveTo(0, 0);
        dragger.lineTo(60, 0);
        dragger.lineTo(60, 60);
        dragger.lineTo(0, 60);
        dragger.lineTo(0, 0);
        dragger.endFill();
        dragger._x = 10; dragger._y = 10;

        var target:MovieClip = _root.createEmptyMovieClip("target", 2);
        target.beginFill(0x33CC66, 100);
        target.moveTo(0, 0);
        target.lineTo(80, 0);
        target.lineTo(80, 80);
        target.lineTo(0, 80);
        target.lineTo(0, 0);
        target.endFill();
        target._x = 110; target._y = 30;

        target.onRollOver = function() { trace("OVER"); };
        target.onRollOut  = function() { trace("OUT");  };

        dragger.onPress = function() {
            this.startDrag(false);
            trace("startDrag");
        };
        dragger.onRelease = dragger.onReleaseOutside = function() {
            this.stopDrag();
            trace("stopDrag");
        };
    }
}
