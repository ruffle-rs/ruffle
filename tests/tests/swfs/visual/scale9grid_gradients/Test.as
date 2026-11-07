package {
    import flash.display.GradientType;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Matrix;
    import flash.geom.Rectangle;

    // A gradient fill moves by the affine its own path's bounds grew by, not by the object's
    // scale: a path spanning the whole shape paints as if there were no grid, and one that
    // only covers part of it gets a two-point fit. Hard stops rather than ramps, so the
    // comparison does not depend on interpolation.
    [SWF(width="380", height="790", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const STOPS:Array = [0, 84, 85, 169, 170, 255];
        private static const ALPHAS:Array = [1, 1, 1, 1, 1, 1];

        public function Test() {
            // Radial and focal, gridded above ungridded, at 3x.
            radialPair(0, 0, true);
            radialPair(1, 0, false);
            radialPair(2, 0.8, false);

            // Linear at 3x0.8: spanning gridded, straddling gridded, spanning ungridded.
            // The two spanning rows must match.
            add(linear(false), 460, true);
            add(linear(true), 570, true);
            add(linear(false), 680, false);
        }

        private function radialPair(index:int, focal:Number, spanning:Boolean):void {
            var y:Number = 20 + index * 140;
            for (var i:int = 0; i < 2; i++) {
                var art:Shape = radial(focal, spanning);
                if (i == 0) {
                    art.scale9Grid = new Rectangle(20, 20, 20, 20);
                }
                art.scaleX = 3;
                art.x = 20;
                art.y = y + i * 70;
                addChild(art);
            }
        }

        private function radial(focal:Number, spanning:Boolean):Shape {
            var s:Shape = new Shape();
            var width:Number = spanning ? 60 : 40;
            var m:Matrix = new Matrix();
            m.createGradientBox(width, 60, 0, 0, 0);
            s.graphics.beginGradientFill(GradientType.RADIAL,
                [0xFF4040, 0xFF4040, 0x4040FF, 0x4040FF, 0xFFFFFF, 0xFFFFFF],
                ALPHAS, STOPS, m, "pad", "rgb", focal);
            s.graphics.drawRect(0, 0, width, 60);
            s.graphics.endFill();
            if (!spanning) {
                // Keeps the bounds at full width so the gradient path covers only part.
                s.graphics.beginFill(0x208020);
                s.graphics.drawRect(48, 0, 12, 60);
                s.graphics.endFill();
            }
            return s;
        }

        private function add(art:Shape, y:Number, grid:Boolean):void {
            if (grid) {
                art.scale9Grid = new Rectangle(30, 30, 60, 60);
            }
            art.scaleX = 3;
            art.scaleY = 0.8;
            art.x = 10;
            art.y = y;
            addChild(art);
        }

        private function linear(straddling:Boolean):Shape {
            var s:Shape = new Shape();
            var width:Number = straddling ? 80 : 120;
            var m:Matrix = new Matrix();
            m.createGradientBox(width, 120, 0, 0, 0);
            s.graphics.beginGradientFill(GradientType.LINEAR,
                [0xFF3030, 0xFF3030, 0x3030FF, 0x3030FF, 0xFFFFFF, 0xFFFFFF],
                ALPHAS, STOPS, m);
            s.graphics.drawRect(0, 0, width, 120);
            s.graphics.endFill();
            if (straddling) {
                s.graphics.beginFill(0x208020);
                s.graphics.drawRect(90, 0, 30, 120);
                s.graphics.endFill();
            }
            return s;
        }
    }
}
