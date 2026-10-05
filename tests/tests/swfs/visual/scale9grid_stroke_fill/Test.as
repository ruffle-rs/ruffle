package {
    import flash.display.GradientType;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Matrix;
    import flash.geom.Rectangle;

    // Where a stroke's gradient lands under slicing. Flash Player anchors the fill on the
    // remapped path bounds widened by half the stroke width, mapped from the authored
    // bounds widened by three eighths of it -- both constants measured, and the hard-stop
    // bands make a wrong constant show as shifted boundaries. Rows vary the width, the
    // inset (the last two put the path in the centre band and across a grid line, pinning
    // that the map applies to the path bounds before the widening), all at 3x.
    [SWF(width="210", height="370", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const GRID:Rectangle = new Rectangle(20, 20, 20, 20);
        private static const COLORS:Array = [0xFF0000, 0xFF0000, 0xFFFF00, 0xFFFF00,
                                             0x00FF00, 0x00FF00, 0x00FFFF, 0x00FFFF,
                                             0x0000FF, 0x0000FF];
        private static const ALPHAS:Array = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        private static const RATIOS:Array = [0, 51, 51, 102, 102, 153, 153, 204, 204, 255];

        public function Test() {
            var rows:Array = [[2, 8], [8, 8], [16, 12], [8, 25], [8, 22]];
            for (var i:int = 0; i < rows.length; i++) {
                var art:Shape = paint(rows[i][0], rows[i][1]);
                art.scale9Grid = GRID;
                art.scaleX = 3;
                art.x = 10;
                art.y = 10 + i * 70;
                addChild(art);
            }
        }

        private function paint(w:Number, inset:Number):Shape {
            var art:Shape = new Shape();
            art.graphics.beginFill(0x101010);
            art.graphics.drawRect(0, 0, 60, 60);
            art.graphics.endFill();
            var m:Matrix = new Matrix();
            m.createGradientBox(60, 60, 0, 0, 0);
            art.graphics.lineStyle(w, 0, 1, false, "normal", "none");
            art.graphics.lineGradientStyle(GradientType.LINEAR, COLORS, ALPHAS, RATIOS, m);
            art.graphics.drawRect(inset, 10, 60 - 2 * inset, 40);
            return art;
        }
    }
}
