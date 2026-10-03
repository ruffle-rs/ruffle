package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Rectangle;

    // A cubic becomes quadratics before the map runs, not after: the map is piecewise linear,
    // so the two orders do not commute and Flash Player only ever remaps quadratics. The
    // count follows the third difference d = P3 - 3P2 + 3P1 - P0: the smallest power of two
    // with |d| <= 4px * k^3. One column per regime -- a degree-elevated quadratic stays a
    // single quadratic, |d| = 30 splits in two, |d| = 52 in four.
    [SWF(width="620", height="170", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const SIZE:Number = 60;

        public function Test() {
            column(20, 20, 0, 40, 0);
            column(220, 8, 0, 18, 0);
            column(420, 8, 0, 14, 10);
        }

        private function column(x:Number, c1x:Number, c1y:Number, c2x:Number, c2y:Number):void {
            var gridded:Shape = cubic(c1x, c1y, c2x, c2y);
            gridded.scale9Grid = new Rectangle(20, 20, 20, 20);
            gridded.scaleX = 3;
            gridded.x = x;
            gridded.y = 20;
            addChild(gridded);

            var plain:Shape = cubic(c1x, c1y, c2x, c2y);
            plain.scaleX = 3;
            plain.x = x;
            plain.y = 90;
            addChild(plain);
        }

        // Fill only: no stroke, so nothing here depends on Ruffle's stroke rasterisation.
        private function cubic(c1x:Number, c1y:Number, c2x:Number, c2y:Number):Shape {
            var s:Shape = new Shape();
            s.graphics.beginFill(0x60C060);
            s.graphics.moveTo(0, SIZE);
            s.graphics.cubicCurveTo(c1x, c1y, c2x, c2y, SIZE, SIZE);
            s.graphics.endFill();
            return s;
        }
    }
}
