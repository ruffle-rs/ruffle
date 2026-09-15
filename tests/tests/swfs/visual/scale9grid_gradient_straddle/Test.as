package {
    import flash.display.GradientType;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Matrix;
    import flash.geom.Rectangle;

    // A gradient-filled path crossing the grid lines: the fill follows the affine its own
    // path's bounds grew by, not the object's scale. A path spanning the whole shape paints
    // as if there were no grid, so the two spanning rows must match.
    [SWF(width="380", height="340", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const SIZE:Number = 120;
        private static const INSET:Number = 30;

        public function Test() {
            add(spanning(), 10, 10, true);
            add(straddling(), 10, 120, true);
            add(spanning(), 10, 230, false);
        }

        private function add(art:Shape, x:Number, y:Number, grid:Boolean):void {
            if (grid) {
                art.scale9Grid =
                    new Rectangle(INSET, INSET, SIZE - 2 * INSET, SIZE - 2 * INSET);
            }
            art.scaleX = 3;
            art.scaleY = 0.8;
            art.x = x;
            art.y = y;
            addChild(art);
        }

        // Hard stops rather than a ramp, so the comparison does not depend on interpolation.
        private function paint(target:Shape, left:Number, width:Number):void {
            var m:Matrix = new Matrix();
            m.createGradientBox(width, SIZE, 0, left, 0);
            target.graphics.beginGradientFill(
                GradientType.LINEAR,
                [0xFF3030, 0xFF3030, 0x3030FF, 0x3030FF, 0xFFFFFF, 0xFFFFFF],
                [1, 1, 1, 1, 1, 1],
                [0, 84, 85, 169, 170, 255],
                m);
            target.graphics.drawRect(left, 0, width, SIZE);
            target.graphics.endFill();
        }

        private function spanning():Shape {
            var s:Shape = new Shape();
            paint(s, 0, SIZE);
            return s;
        }

        private function straddling():Shape {
            var s:Shape = new Shape();
            paint(s, 0, SIZE * 2 / 3);
            // Keeps the bounds at full width so the grid stays valid and the path straddles.
            s.graphics.beginFill(0x208020);
            s.graphics.drawRect(SIZE - INSET, 0, INSET, SIZE);
            s.graphics.endFill();
            return s;
        }
    }
}
