// compiled with mxmlc

package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.geom.Rectangle;

    // A stroke inside the stretched band keeps its authored width rather than growing with
    // the region it sits in.
    public class Test extends MovieClip {
        private static const SIZE:Number = 60;
        private static const GRID:Rectangle = new Rectangle(20, 20, 20, 20);

        public function Test() {
            row(0, stroked);
            row(1, filledAndStroked);
        }

        // No plain reference copy next to these: an ungridded stroke under scaleX=3 hits
        // Flash Player's non-uniform stroke-width scaling, which Ruffle draws differently
        // for reasons that have nothing to do with the grid.
        private function row(index:int, paint:Function):void {
            var y:Number = 20 + index * 70;

            var gridded:Shape = paint();
            gridded.scale9Grid = GRID;
            gridded.scaleX = 3;
            gridded.x = 20;
            gridded.y = y;
            addChild(gridded);
        }

        private function stroked():Shape {
            var s:Shape = new Shape();
            s.graphics.lineStyle(8, 0x2060C0);
            s.graphics.drawRect(0, 0, SIZE, SIZE);
            s.graphics.lineStyle(4, 0xC02060);
            s.graphics.moveTo(20, 0);
            s.graphics.lineTo(20, SIZE);
            s.graphics.moveTo(40, 0);
            s.graphics.lineTo(40, SIZE);
            return s;
        }

        private function filledAndStroked():Shape {
            var s:Shape = new Shape();
            s.graphics.lineStyle(6, 0x102040);
            s.graphics.beginFill(0x40A0E0);
            s.graphics.drawRect(0, 0, SIZE, SIZE);
            s.graphics.endFill();
            return s;
        }
    }
}
