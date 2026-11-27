// Calls BitmapData.generateFilterRect with drop shadow and bevel filters at an
// angle of 90 degrees and large negative distances, so that their offset copy
// is moved up. avm2/bitmapdata_generatefilterrect covers the same distances to
// the left.
package {
    import flash.display.BitmapData;
    import flash.display.Sprite;
    import flash.filters.BevelFilter;
    import flash.filters.DropShadowFilter;
    import flash.geom.Rectangle;

    public class Test extends Sprite {
        public function Test() {
            var bitmap:BitmapData = new BitmapData(100, 80, true, 0);
            var source:Rectangle = new Rectangle(10, 10, 25, 25);

            for each (var distance:Number in [-255.5, -256, -256.004, -261.796, -1000.5]) {
                var shadow:DropShadowFilter = new DropShadowFilter(distance, 90, 0, 1, 0, 0, 1, 1);
                trace("drop shadow, distance " + distance + ": " + bitmap.generateFilterRect(source, shadow));

                var bevel:BevelFilter = new BevelFilter(distance, 90, 0xFFFFFF, 1, 0, 1, 0, 0, 1, 1, "outer");
                trace("bevel, distance " + distance + ": " + bitmap.generateFilterRect(source, bevel));
            }

            trace("Done");
        }
    }
}
