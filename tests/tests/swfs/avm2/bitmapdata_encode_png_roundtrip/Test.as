package {
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.Loader;
    import flash.display.MovieClip;
    import flash.display.PNGEncoderOptions;
    import flash.events.Event;
    import flash.events.IOErrorEvent;
    import flash.geom.Rectangle;
    import flash.utils.ByteArray;

    public class Test extends MovieClip {
        private var tests:Array;
        private var idx:int = 0;

        public function Test() {
            tests = [
                {label: "32x24 varying alpha (default compression)", make: makeVaryingAlpha, opts: new PNGEncoderOptions()},
                {label: "32x24 varying alpha (fastCompression=true)", make: makeVaryingAlpha, opts: new PNGEncoderOptions(true)},
                {label: "32x24 opaque (default)", make: makeOpaque, opts: new PNGEncoderOptions()},
                {label: "32x24 opaque (fastCompression=true)", make: makeOpaque, opts: new PNGEncoderOptions(true)},
                {label: "1x1 opaque red pixel", make: makeOnePixel, opts: new PNGEncoderOptions()},
                {label: "transparent fully zero alpha", make: makeAllZeroAlpha, opts: new PNGEncoderOptions()}
            ];
            runNext();
        }

        private function runNext():void {
            if (idx >= tests.length) {
                trace("");
                trace("// done");
                return;
            }
            var t:Object = tests[idx++];
            trace("");
            trace("// " + t.label);
            var orig:BitmapData = t.make();
            var encoded:ByteArray = orig.encode(orig.rect, t.opts);
            trace("encoded length > 0: " + (encoded.length > 0));
            encoded.position = 0;
            var loader:Loader = new Loader();
            loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
                var content:Object = loader.content;
                if (!(content is Bitmap)) {
                    trace("loaded content not Bitmap: " + content);
                    runNext();
                    return;
                }
                var loaded:BitmapData = Bitmap(content).bitmapData;
                trace("loaded w=" + loaded.width + " h=" + loaded.height);
                var result:Object = orig.compare(loaded);
                trace("compare == 0: " + (result === 0));
                if (result !== 0) {
                    trace("compare result type: " + (result is BitmapData ? "BitmapData" : "other"));
                    if (result is Number || result is int) trace("compare result: " + result);
                }
                runNext();
            });
            loader.contentLoaderInfo.addEventListener(IOErrorEvent.IO_ERROR, function(e:IOErrorEvent):void {
                trace("io error: " + e.text);
                runNext();
            });
            loader.loadBytes(encoded);
        }

        private static function makeVaryingAlpha():BitmapData {
            var bd:BitmapData = new BitmapData(32, 24, true, 0x00000000);
            for (var y:int = 0; y < 24; y++) {
                for (var x:int = 0; x < 32; x++) {
                    var a:int = (x * 8) & 0xFF;
                    var r:int = (y * 10) & 0xFF;
                    var g:int = ((x + y) * 6) & 0xFF;
                    var b:int = ((x * y) % 256) & 0xFF;
                    bd.setPixel32(x, y, (a << 24) | (r << 16) | (g << 8) | b);
                }
            }
            return bd;
        }

        private static function makeOpaque():BitmapData {
            var bd:BitmapData = new BitmapData(32, 24, false, 0xFF000000);
            for (var y:int = 0; y < 24; y++) {
                for (var x:int = 0; x < 32; x++) {
                    var r:int = (x * 8) & 0xFF;
                    var g:int = (y * 10) & 0xFF;
                    var b:int = ((x + y) * 6) & 0xFF;
                    bd.setPixel32(x, y, (0xFF << 24) | (r << 16) | (g << 8) | b);
                }
            }
            return bd;
        }

        private static function makeOnePixel():BitmapData {
            return new BitmapData(1, 1, false, 0xFFFF0000);
        }

        private static function makeAllZeroAlpha():BitmapData {
            return new BitmapData(16, 16, true, 0x00000000);
        }
    }
}
