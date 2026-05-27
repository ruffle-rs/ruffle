package {
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.display.PNGEncoderOptions;
    import flash.utils.ByteArray;

    [SWF(width="540", height="100", backgroundColor="0x808080")]
    public class Test extends MovieClip {
        public function Test() {
            var src:BitmapData = makeGradient(60, 60);
            placeOriginal(src, 20, 20);
            placeEncoded(src, 100, 20, false);
            placeEncoded(src, 180, 20, true);
        }
        private function placeOriginal(bd:BitmapData, dx:int, dy:int):void {
            var b:Bitmap = new Bitmap(bd);
            b.x = dx;
            b.y = dy;
            addChild(b);
        }
        private function placeEncoded(bd:BitmapData, dx:int, dy:int, fast:Boolean):void {
            var encoded:ByteArray = bd.encode(bd.rect, new PNGEncoderOptions(fast));
            encoded.position = 0;
            var loader:Loader = new Loader();
            loader.x = dx;
            loader.y = dy;
            addChild(loader);
            loader.loadBytes(encoded);
        }
        private static function makeGradient(w:int, h:int):BitmapData {
            var bd:BitmapData = new BitmapData(w, h, true, 0x00000000);
            for (var y:int = 0; y < h; y++) {
                for (var x:int = 0; x < w; x++) {
                    var a:int = 128 + ((x * 4) & 0x7F);
                    var r:int = Math.floor(255 * x / (w - 1));
                    var g:int = Math.floor(255 * y / (h - 1));
                    var b:int = Math.floor(255 * (x + y) / (w + h - 2));
                    bd.setPixel32(x, y, (a << 24) | (r << 16) | (g << 8) | b);
                }
            }
            return bd;
        }
    }
}
