package {
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.geom.Matrix;
    import flash.geom.Rectangle;

    // A 3x3 mosaic whose cell edges lie on the grid and bounds lines, stretched unevenly.
    // The corners carry non-repeating bitmap fills, so a fill matrix that fails to travel
    // with its path shows up as a crop or a phase shift. Every edge lands on a whole pixel,
    // so there is no antialiasing to make the comparison fragile.
    [SWF(width="260", height="380", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const SIZE:Number = 100;
        private static const INSET:Number = 20;

        public function Test() {
            var art:Shape = new Shape();
            paint(art);
            art.scale9Grid = new Rectangle(INSET, INSET, SIZE - 2 * INSET, SIZE - 2 * INSET);
            art.scaleX = 2.4;
            art.scaleY = 1.7;
            art.x = 10;
            art.y = 10;
            addChild(art);

            var plain:Shape = new Shape();
            paint(plain);
            plain.scaleX = 2.4;
            plain.scaleY = 1.7;
            plain.x = 10;
            plain.y = 200;
            addChild(plain);
        }

        private function paint(target:Shape):void {
            var solids:Array = [
                [0xE04040, 0x40E040, 0x4040E0],
                [0xE0E040, 0xFFFFFF, 0x40E0E0],
                [0xE040E0, 0x808080, 0x203060]];

            for (var iy:int = 0; iy < 3; iy++) {
                for (var ix:int = 0; ix < 3; ix++) {
                    var x:Number = ix == 0 ? 0 : (ix == 1 ? INSET : SIZE - INSET);
                    var y:Number = iy == 0 ? 0 : (iy == 1 ? INSET : SIZE - INSET);
                    var w:Number = ix == 1 ? SIZE - 2 * INSET : INSET;
                    var h:Number = iy == 1 ? SIZE - 2 * INSET : INSET;

                    var corner:Boolean = (ix != 1) && (iy != 1);
                    if (corner) {
                        var m:Matrix = new Matrix();
                        m.translate(x, y);
                        target.graphics.beginBitmapFill(quadrants(ix, iy), m, false, false);
                    } else {
                        target.graphics.beginFill(solids[iy][ix]);
                    }
                    target.graphics.drawRect(x, y, w, h);
                    target.graphics.endFill();
                }
            }
        }

        private function quadrants(ix:int, iy:int):BitmapData {
            var base:uint = uint(0x300000 * (ix + 1) + 0x003000 * (iy + 1) + 0x40);
            var bd:BitmapData = new BitmapData(INSET, INSET, false, base);
            bd.fillRect(new Rectangle(0, 0, 12, 12), base + 0x606060);
            bd.fillRect(new Rectangle(12, 0, 8, 6), base + 0x202020);
            bd.fillRect(new Rectangle(0, 12, 5, 8), base + 0xA0A0A0);
            bd.setPixel(INSET - 1, INSET - 1, 0xFFFFFF);
            bd.setPixel(0, 0, 0x000000);
            return bd;
        }
    }
}
