package {
    import flash.display.Sprite;
    import flash.display.BitmapData;
    import flash.geom.Rectangle;
    import flash.geom.Point;

    // copyPixels with alphaBitmapData, exhaustive: result alpha for every
    // (source alpha, alpha bitmap alpha) pair, one row per source alpha.
    // Dest starts fully transparent with mergeAlpha=false, so the written
    // alpha is the combine function's exact output.
    public class Test extends Sprite {
        public function Test() {
            var rect:Rectangle = new Rectangle(0, 0, 1, 1);
            var origin:Point = new Point(0, 0);
            var src:BitmapData = new BitmapData(1, 1, true, 0);
            var alp:BitmapData = new BitmapData(1, 1, true, 0);
            var dst:BitmapData = new BitmapData(1, 1, true, 0);
            var hash:Number = 5381;
            var edges:Array = [0, 1, 2, 127, 128, 129, 253, 254, 255];
            var grid:Object = {};
            for (var sa:int = 0; sa < 256; sa++) {
                var row:String = "";
                for (var aa:int = 0; aa < 256; aa++) {
                    src.setPixel32(0, 0, ((sa << 24) | 0x00FF00) >>> 0);
                    alp.setPixel32(0, 0, (aa << 24) >>> 0);
                    dst.setPixel32(0, 0, 0);
                    dst.copyPixels(src, rect, origin, alp, origin, false);
                    var out:uint = dst.getPixel32(0, 0);
                    hash = (hash * 33 + (out >>> 24)) % 4294967291;
                    hash = (hash * 33 + ((out >>> 8) & 0xFF)) % 4294967291;
                    if (edges.indexOf(sa) >= 0 && edges.indexOf(aa) >= 0) {
                        row += " " + out.toString(16);
                    }
                }
                if (row.length > 0) {
                    grid[sa] = row;
                }
            }
            trace("sweep hash: " + hash);
            for each (var e:int in edges) {
                trace("sa=" + e + ":" + grid[e]);
            }

            // Toxic 2 shape: green over opaque terrain, dest as its own alpha
            // source, mergeAlpha on, then an exact-match threshold cuts holes.
            var ground:BitmapData = new BitmapData(3, 1, true, 0xFF663311);
            ground.setPixel32(2, 0, 0x00000000);
            var hole:BitmapData = new BitmapData(3, 1, true, 0xFF00FF00);
            ground.copyPixels(hole, new Rectangle(0, 0, 3, 1), origin, ground, origin, true);
            var line3:String = "toxic stamp:";
            var i:int;
            for (i = 0; i < 3; i++) {
                line3 += " " + ground.getPixel32(i, 0).toString(16);
            }
            trace(line3);
            var n:uint = ground.threshold(ground, new Rectangle(0, 0, 3, 1), origin, "==", 0xFF00FF00, 0, 0xFFFFFF, false);
            line3 = "toxic thresh n=" + n + ":";
            for (i = 0; i < 3; i++) {
                line3 += " " + ground.getPixel32(i, 0).toString(16);
            }
            trace(line3);
            trace("done");
        }
    }
}
