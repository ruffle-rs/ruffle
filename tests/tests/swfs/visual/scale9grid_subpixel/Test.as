package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Rectangle;

    // The grid is stored to the twip and only truncated when read back, so sub-pixel
    // grids render differently. The red/blue edge sits in the middle band at x=30; at 3x
    // it lands at 90, 89.24 and 87.66 for a grid starting at 20, 20.25 and 20.75. Were the
    // grid truncated when set, all three rows would match.
    [SWF(width="200", height="100", backgroundColor="#101010")]
    public class Test extends MovieClip {
        public function Test() {
            var starts:Array = [20, 20.25, 20.75];
            for (var i:int = 0; i < starts.length; i++) {
                var s:Shape = new Shape();
                s.graphics.beginFill(0xD04040);
                s.graphics.drawRect(0, 0, 30, 60);
                s.graphics.endFill();
                s.graphics.beginFill(0x4040D0);
                s.graphics.drawRect(30, 0, 30, 60);
                s.graphics.endFill();
                s.scale9Grid = new Rectangle(starts[i], 20, 40 - starts[i], 20);
                s.scaleX = 3;
                s.scaleY = 0.4;
                s.x = 10;
                s.y = 10 + i * 30;
                addChild(s);
            }
        }
    }
}
