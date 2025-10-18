class Test {
    static function main() {
        var isDown:Boolean = false;

        // Two adjacent clips. Drag without startDrag should trigger onDragOver on the hovered clip.
        var left:MovieClip = _root.createEmptyMovieClip("left", 1);
        left.beginFill(0xCC3333, 100);
        left.moveTo(0, 0);
        left.lineTo(50, 0);
        left.lineTo(50, 50);
        left.lineTo(0, 50);
        left.lineTo(0, 0);
        left.endFill();
        left._x = 10; left._y = 10;
        left.trackAsMenu = true;
        left.useHandCursor = false;
        left.onPress = function() { isDown = true; };
        left.onRelease = function() { isDown = false; };

        var right:MovieClip = _root.createEmptyMovieClip("right", 2);
        right.beginFill(0x33CC66, 100);
        right.moveTo(0, 0);
        right.lineTo(50, 0);
        right.lineTo(50, 50);
        right.lineTo(0, 50);
        right.lineTo(0, 0);
        right.endFill();
        right._x = 80; right._y = 10;
        right.trackAsMenu = true;
        right.useHandCursor = false;
        right.onPress = function() { isDown = true; };
        right.onRelease = function() { isDown = false; };

        right.onDragOver = function() {
            if (isDown) {
                trace("OK");
                fscommand("quit");
            }
        };

        // Safety: ensure mouse up anywhere clears the flag
        var mouseListener:Object = {};
        mouseListener.onMouseUp = function() { isDown = false; };
        Mouse.addListener(mouseListener);
    }
}
