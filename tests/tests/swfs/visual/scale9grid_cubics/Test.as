// compiled with mxmlc

package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Rectangle;

    // A cubic becomes quadratics before the map runs, not after: the map is piecewise linear,
    // so the two orders do not commute and Flash Player only ever remaps quadratics.
    public class Test extends MovieClip {
        private static const SIZE:Number = 60;

        public function Test() {
            var gridded:Shape = cubic();
            gridded.scale9Grid = new Rectangle(20, 20, 20, 20);
            gridded.scaleX = 3;
            gridded.x = 20;
            gridded.y = 20;
            addChild(gridded);

            var plain:Shape = cubic();
            plain.scaleX = 3;
            plain.x = 20;
            plain.y = 90;
            addChild(plain);
        }

        // Fill only: no stroke, so nothing here depends on Ruffle's stroke rasterisation.
        private function cubic():Shape {
            var s:Shape = new Shape();
            s.graphics.beginFill(0x60C060);
            s.graphics.moveTo(0, SIZE);
            s.graphics.cubicCurveTo(20, 0, 40, 0, SIZE, SIZE);
            s.graphics.endFill();
            return s;
        }
    }
}
