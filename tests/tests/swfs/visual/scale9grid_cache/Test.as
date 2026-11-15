package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Rectangle;

    // Several instances of one character, same grid, different geometry: a cache key that
    // missed the geometry would serve the first instance's slicing to all of them.
    [SWF(width="280", height="480", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const SIZE:Number = 100;
        private static const GRID:Rectangle = new Rectangle(25, 25, 50, 50);

        public function Test() {
            for (var i:int = 0; i < 4; i++) {
                var t:Number = i / 3;
                var art:Shape = paint(t);
                art.scale9Grid = GRID;
                art.scaleX = 2.4;
                art.scaleY = 1.0;
                art.x = 12;
                art.y = 12 + i * 115;
                addChild(art);
            }
        }

        // The full-size background holds bounds fixed while the corner blocks shrink with t.
        private function paint(t:Number):Shape {
            var s:Shape = new Shape();
            var r:Number = 25 - 12 * t;

            s.graphics.beginFill(0x2050A0);
            s.graphics.drawRect(0, 0, SIZE, SIZE);
            s.graphics.endFill();

            s.graphics.beginFill(0xF0C030);
            s.graphics.drawRect(0, 0, r, r);
            s.graphics.drawRect(SIZE - r, 0, r, r);
            s.graphics.drawRect(0, SIZE - r, r, r);
            s.graphics.drawRect(SIZE - r, SIZE - r, r, r);
            s.graphics.endFill();

            s.graphics.beginFill(0xE04060);
            s.graphics.drawRect(40, 40 - 10 * t, 20, 20 + 20 * t);
            s.graphics.endFill();

            return s;
        }
    }
}
