// Tests which channel a DisplacementMapFilter component picks when it's
// drawn, and that the component survives a round trip through
// MovieClip.filters, for the component values in the list below.

import flash.display.BitmapData;
import flash.filters.DisplacementMapFilter;
import flash.geom.Point;

var values = [0, 1, 2, 3, 4, 5, 8, 9, 16, 17, 18, 24, 255, 256, 257, 258, 260, 264, 513, -1, -255, 4294967295, 1.5, NaN];

// Each pixel's color is its own position, so the drawn pixel shows where it
// was taken from.
var source = new BitmapData(40, 40, false, 0);
for (var y = 0; y < 40; y++) {
    for (var x = 0; x < 40; x++) {
        source.setPixel(x, y, x << 16 | y << 8);
    }
}

// An opaque map with a scale of 21: red moves a pixel by about +5.3, green by
// -7.9, blue by -2.6 and alpha by +10.4 pixels.
var map = new BitmapData(40, 40, true, 0xffc02060);

// Returns where the pixel in the middle was taken from, along the axis that the
// component is used for.
function sampled(component, vertical) {
    var componentX = vertical ? 0 : component;
    var componentY = vertical ? component : 0;
    var out = new BitmapData(40, 40, false, 0xffffff);
    var filter = new DisplacementMapFilter(map, new Point(0, 0), componentX, componentY, 21, 21, "clamp");
    out.applyFilter(source, source.rectangle, new Point(0, 0), filter);
    var pixel = out.getPixel(20, 20);

    return vertical ? (pixel >> 8) & 0xff : pixel >> 16;
}

trace("// Drawn with applyFilter, as componentX and as componentY");
for (var i = 0; i < values.length; i++) {
    trace(values[i] + ": x " + sampled(values[i], false) + ", y " + sampled(values[i], true));
}

trace("// Read back from MovieClip.filters");
var mc = _root.createEmptyMovieClip("mc", 1);
for (var i = 0; i < values.length; i++) {
    var c = values[i];
    mc.filters = [new DisplacementMapFilter(map, new Point(0, 0), c, c, 21, 21, "clamp")];
    var filter = mc.filters[0];
    trace(c + ": " + filter.componentX + ", " + filter.componentY);
}
