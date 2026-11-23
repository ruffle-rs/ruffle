package {
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.Shader;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.events.IOErrorEvent;
    import flash.filters.ShaderFilter;
    import flash.geom.Point;
    import flash.net.URLLoader;
    import flash.net.URLLoaderDataFormat;
    import flash.net.URLRequest;

    // Applies the ordered dithering shader from avm2/pixelbender_dithering
    // (orderedDither.pbk) to every gray level at every position of its
    // dithering matrix, for a few alphas and matrix sizes.
    //
    // For each matrix row and column, it traces the lowest gray level that the
    // shader turns white, and whether every higher level is white too.
    //
    // It also shows the shader as a filter on the stage with the default 4x4
    // matrix: four strips of 64 gray levels, each level four pixels wide, with
    // four rows of full alpha above four rows of half alpha, so that every
    // level meets every matrix position.
    [SWF(width = "256", height = "32")]
    public class Test extends Sprite {
        private static const ALPHAS:Array = [255, 128];

        public function Test() {
            var loader:URLLoader = new URLLoader();
            loader.dataFormat = URLLoaderDataFormat.BINARY;
            loader.addEventListener(Event.COMPLETE, function(e:Event):void {
                var shader:Shader = new Shader(loader.data);

                // With an odd width, some thresholds equal a gray level exactly, and
                // which way those go presumably depends on the precision of the GPU.
                testMatrix(shader, 4, 4);
                testMatrix(shader, 6, 3);

                showOnStage(shader);
            });
            loader.addEventListener(IOErrorEvent.IO_ERROR, function(e:IOErrorEvent):void {
                trace("Failed to load the shader: " + e.text);
            });
            loader.load(new URLRequest("orderedDither.pbj"));
        }

        private function showOnStage(shader:Shader):void {
            shader.data.matrixWidth.value = [4];
            shader.data.matrixHeight.value = [4];

            var source:BitmapData = new BitmapData(256, 32, true, 0);

            for (var y:int = 0; y < 32; y++) {
                var strip:int = y / 8;
                var alpha:uint = (y % 8) < 4 ? 255 : 128;

                for (var x:int = 0; x < 256; x++) {
                    var level:uint = strip * 64 + x / 4;
                    source.setPixel32(x, y, alpha << 24 | level << 16 | level << 8 | level);
                }
            }

            var bitmap:Bitmap = new Bitmap(source);
            bitmap.filters = [new ShaderFilter(shader)];
            addChild(bitmap);
        }

        private function testMatrix(shader:Shader, width:int, height:int):void {
            shader.data.matrixWidth.value = [width];
            shader.data.matrixHeight.value = [height];
            trace("// " + width + "x" + height + " matrix");

            for each (var alpha:int in ALPHAS) {
                // Each gray level is repeated once for every matrix column, and the
                // rows are repeated once for every matrix row.
                var source:BitmapData = new BitmapData(256 * width, height, true, 0);

                for (var y:int = 0; y < height; y++) {
                    for (var x:int = 0; x < 256 * width; x++) {
                        var level:uint = x / width;
                        var color:uint = uint(alpha) << 24 | level << 16 | level << 8 | level;
                        source.setPixel32(x, y, color);
                    }
                }

                var result:BitmapData = new BitmapData(source.width, source.height, true, 0);
                result.applyFilter(source, source.rect, new Point(0, 0), new ShaderFilter(shader));

                for (y = 0; y < height; y++) {
                    var thresholds:Array = [];
                    var monotonic:Boolean = true;

                    for (var column:int = 0; column < width; column++) {
                        var threshold:* = "none";
                        var white:Boolean = false;

                        for (var l:int = 0; l < 256; l++) {
                            var pixel:uint = result.getPixel32(l * width + column, y);
                            var isWhite:Boolean = pixel == 0xffffffff;

                            if (!isWhite && pixel != 0xff000000) {
                                trace("Unexpected pixel " + pixel.toString(16) + " at level " + l);
                            }

                            if (isWhite && threshold == "none") {
                                threshold = l;
                            }

                            if (white && !isWhite) {
                                monotonic = false;
                            }

                            white = isWhite;
                        }

                        thresholds.push(threshold);
                    }

                    var suffix:String = monotonic ? "" : " (not monotonic)";
                    trace("alpha " + alpha + ", row " + y + ": " + thresholds.join(" ") + suffix);
                }
            }
        }
    }
}
