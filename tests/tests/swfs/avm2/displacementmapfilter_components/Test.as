package {
    import flash.display.BitmapData;
    import flash.display.Sprite;
    import flash.filters.DisplacementMapFilter;
    import flash.geom.Point;

    // Tests which channel a DisplacementMapFilter component picks when it's
    // drawn, and that the component survives a round trip through
    // DisplayObject.filters, for the component values in VALUES.
    public class Test extends Sprite {
        private static const VALUES:Array = [
            0, 1, 2, 3, 4, 5, 8, 9, 15, 16, 17, 18, 20, 24, 255, 256, 257, 258, 260, 264, 513,
            4294967295, 4294967041
        ];

        public function Test() {
            // Each pixel's color is its own position, so the drawn pixel
            // shows where it was taken from.
            var source:BitmapData = new BitmapData(40, 40, false, 0);
            for (var y:int = 0; y < 40; y++) {
                for (var x:int = 0; x < 40; x++) {
                    source.setPixel(x, y, x << 16 | y << 8);
                }
            }

            // An opaque map with a scale of 21: red moves a pixel by about
            // +5.3, green by -7.9, blue by -2.6 and alpha by +10.4 pixels.
            var map:BitmapData = new BitmapData(40, 40, true, 0xffc02060);

            trace("// Drawn with applyFilter, as componentX and as componentY");
            var c:uint;
            for each (c in VALUES) {
                trace(c + ": x " + sampled(source, map, c, false) + ", y " + sampled(source, map, c, true));
            }

            trace("// Read back from DisplayObject.filters");
            var sprite:Sprite = new Sprite();
            for each (c in VALUES) {
                sprite.filters = [new DisplacementMapFilter(map, new Point(0, 0), c, c, 21, 21, "clamp")];
                var filter:DisplacementMapFilter = sprite.filters[0];
                trace(c + ": " + filter.componentX + ", " + filter.componentY);
            }
        }

        // Returns where the pixel in the middle was taken from, along the
        // axis that the component is used for.
        private function sampled(source:BitmapData, map:BitmapData, component:uint, vertical:Boolean):String {
            var componentX:uint = vertical ? 0 : component;
            var componentY:uint = vertical ? component : 0;
            var out:BitmapData = new BitmapData(40, 40, false, 0xffffff);
            var filter:DisplacementMapFilter = new DisplacementMapFilter(map, new Point(0, 0), componentX, componentY, 21, 21, "clamp");
            out.applyFilter(source, source.rect, new Point(0, 0), filter);
            var pixel:uint = out.getPixel(20, 20);

            return vertical ? String((pixel >> 8) & 0xff) : String(pixel >> 16);
        }
    }
}
