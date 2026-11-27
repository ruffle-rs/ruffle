// Tests BitmapData.paletteMap with a movie clip as the red channel array,
// compared with an array. The source pixel is opaque black, and element 0
// of the clip and of the array is 0x00112233. The traced value is the RGB
// value of the result in hexadecimal.

function rgb() {
    return destination.getPixel(0, 0).toString(16);
}

var source = new flash.display.BitmapData(1, 1, true, 0xFF000000);
var destination = new flash.display.BitmapData(1, 1, true, 0);
var origin = new flash.geom.Point(0, 0);

createEmptyMovieClip("redClip", getNextHighestDepth());
redClip[0] = 0x00112233;
destination.paletteMap(source, source.rectangle, origin, redClip, null, null, null);
trace("clip: " + rgb());

redClip.length = 256;
destination.paletteMap(source, source.rectangle, origin, redClip, null, null, null);
trace("clip with length: " + rgb());

destination.paletteMap(source, source.rectangle, origin, [0x00112233], null, null, null);
trace("array: " + rgb());
