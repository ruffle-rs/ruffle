// Maps a pixel with BitmapData.paletteMap into transparent and opaque bitmaps,
// with channel values whose sum has different alpha bytes. Opaque bitmaps keep
// an alpha of 0xFF and the color as is.
package {
    import flash.display.BitmapData;
    import flash.display.Sprite;
    import flash.geom.Point;
    import flash.geom.Rectangle;

    public class Test extends Sprite {
        public function Test() {
            run("transparent", true);
            run("opaque", false);
            trace("Done");
        }

        private function hex(v:uint):String {
            var s:String = v.toString(16).toUpperCase();
            while (s.length < 8) {
                s = "0" + s;
            }
            return "0x" + s;
        }

        private function run(label:String, transparent:Boolean):void {
            var cases:Array = [
                ["zero alpha sum", 0x00112233],
                ["half alpha sum", 0x80112233],
                ["full alpha sum", 0xFF112233],
                ["alpha only", 0x7F000000]
            ];
            for each (var c:Array in cases) {
                var src:BitmapData = new BitmapData(2, 1, transparent, 0xFF102030);
                var dst:BitmapData = new BitmapData(2, 1, transparent, 0xFF000000);
                var red:Array = [];
                for (var i:int = 0; i < 256; i++) {
                    red.push(c[1]);
                }
                var zeros:Array = [];
                for (var j:int = 0; j < 256; j++) {
                    zeros.push(0);
                }
                dst.paletteMap(src, new Rectangle(0, 0, 2, 1), new Point(0, 0), red, zeros, zeros, zeros);
                trace(label + " " + c[0] + ": " + hex(dst.getPixel32(0, 0)));
            }
        }
    }
}
