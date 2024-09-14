package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
    }
}

import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.geom.Point;

var dest = new BitmapData(1, 1, true, 0);
var source = new BitmapData(1, 1, true, 0);

for (var red = 0; red < 256; red += 1) {
    for (var alpha = 0; alpha < 256; alpha += 1) {
        var destPixel:uint = (alpha << 24) | red;
        var tmp = dest.clone();
        tmp.setPixel32(0, 0, destPixel);
        // Account for lossy roundtrip caused by premultiply/unmultiply
        var expected = tmp.getPixel32(0, 0);
        tmp.copyPixels(source, new Rectangle(0, 0, 1, 1), new Point(0, 0), null, null, true);
        var roundTrip = tmp.getPixel32(0, 0);
        if (expected != roundTrip) {
            trace("MISMATCH: destPixel: " + destPixel.toString(16) + " Expected: " + expected.toString(16) + " roundTrip: " + roundTrip.toString(16));
        }
    }
}
trace("Done");