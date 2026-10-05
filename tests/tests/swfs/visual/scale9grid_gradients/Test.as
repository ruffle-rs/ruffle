package {
    import flash.display.GradientType;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Matrix;
    import flash.geom.Rectangle;

    // Radial and focal gradient fills follow the same rule as linear ones: the fill moves by
    // the affine its own path's bounds grew by. Each case is drawn gridded and ungridded.
    [SWF(width="240", height="450", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const SIZE:Number = 60;
        private static const GRID:Rectangle = new Rectangle(20, 20, 20, 20);

        public function Test() {
            pair(0, GradientType.RADIAL, 0, true);
            pair(1, GradientType.RADIAL, 0, false);
            pair(2, GradientType.RADIAL, 0.8, false);
        }

        // `spanning` decides whether the filled path covers the whole shape or only part of
        // it, which is what separates an exact fit from a two-point one.
        private function pair(index:int, type:String, focal:Number, spanning:Boolean):void {
            var y:Number = 20 + index * 140;

            var art:Shape = paint(type, focal, spanning);
            art.scale9Grid = GRID;
            art.scaleX = 3;
            art.x = 20;
            art.y = y;
            addChild(art);

            var plain:Shape = paint(type, focal, spanning);
            plain.scaleX = 3;
            plain.x = 20;
            plain.y = y + 70;
            addChild(plain);
        }

        private function paint(type:String, focal:Number, spanning:Boolean):Shape {
            var s:Shape = new Shape();
            var width:Number = spanning ? SIZE : SIZE * 2 / 3;
            var m:Matrix = new Matrix();
            m.createGradientBox(width, SIZE, 0, 0, 0);
            s.graphics.beginGradientFill(
                type,
                [0xFF4040, 0xFF4040, 0x4040FF, 0x4040FF, 0xFFFFFF, 0xFFFFFF],
                [1, 1, 1, 1, 1, 1],
                [0, 84, 85, 169, 170, 255],
                m, "pad", "rgb", focal);
            s.graphics.drawRect(0, 0, width, SIZE);
            s.graphics.endFill();
            if (!spanning) {
                s.graphics.beginFill(0x208020);
                s.graphics.drawRect(SIZE - 12, 0, 12, SIZE);
                s.graphics.endFill();
            }
            return s;
        }
    }
}
