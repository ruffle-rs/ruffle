package {
    import flash.display.GradientType;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.geom.Matrix;
    import flash.geom.Rectangle;

    // A child sliced by its container's grid has its fill moved by the affine taking the
    // path's own bounds to where the map sends them, expressed back in the child's space.
    // Solid fills cannot show that, so every child here is gradient filled, and the curved
    // row's control points decide the bounds the fit is derived from.
    [SWF(width="420", height="290", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const SIZE:Number = 60;
        private static const GRID:Rectangle = new Rectangle(20, 20, 20, 20);

        public function Test() {
            row(0, straight);
            row(1, curved);
            row(2, offset);
        }

        private function row(index:int, paint:Function):void {
            var y:Number = 20 + index * 80;

            var gridded:Sprite = new Sprite();
            gridded.addChild(paint());
            gridded.scale9Grid = GRID;
            gridded.scaleX = 3;
            gridded.x = 20;
            gridded.y = y;
            addChild(gridded);

            var plain:Sprite = new Sprite();
            plain.addChild(paint());
            plain.scaleX = 3;
            plain.x = 220;
            plain.y = y;
            addChild(plain);
        }

        private function gradient(s:Shape, width:Number):void {
            var m:Matrix = new Matrix();
            m.createGradientBox(width, SIZE);
            s.graphics.beginGradientFill(GradientType.LINEAR,
                [0x2040C0, 0xE0C020], [1, 1], [0, 255], m);
        }

        // Spans the whole width, so the fit crosses all three regions.
        private function straight():Shape {
            var s:Shape = new Shape();
            gradient(s, SIZE);
            s.graphics.drawRect(0, 0, SIZE, SIZE);
            s.graphics.endFill();
            return s;
        }

        // Control points reach past the anchors into the next region, which is what the
        // bounds behind the fill fit have to account for.
        private function curved():Shape {
            var s:Shape = new Shape();
            gradient(s, SIZE);
            s.graphics.moveTo(0, SIZE);
            s.graphics.curveTo(SIZE / 2, -SIZE / 2, SIZE, SIZE);
            s.graphics.lineTo(0, SIZE);
            s.graphics.endFill();
            return s;
        }

        // Carries a translate on both axes, so the grid is only reached by conjugating the
        // fit through the child's matrix.
        private function offset():Shape {
            var s:Shape = new Shape();
            gradient(s, SIZE);
            s.graphics.drawRect(0, 0, SIZE, SIZE);
            s.graphics.endFill();
            s.x = 14;
            s.y = 6;
            return s;
        }
    }
}
