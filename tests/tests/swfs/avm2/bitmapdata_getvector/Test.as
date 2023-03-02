// compiled with mxmlc

// test mostly copied from get_pixels test
package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {}
}

import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.utils.ByteArray;

var WIDTH: uint = 10;
var HEIGHT: uint = 10;

function createSource(): BitmapData {
    var src: BitmapData = new BitmapData(WIDTH, HEIGHT, true, 0x00000000);
    src.noise(0);
    return src;
}

function printPixels(src: BitmapData, x: int, y: int, width: uint, height: uint): void {
    var rect: Rectangle = new Rectangle(x, y, width, height);
    var pixels: Vector.<uint> = src.getVector(rect);

    trace("/// var pixels = getVector(new Rectangle(" + x + ", " + y + ", " + width + ", " + height + "))");

    trace("// pixels.length");
    trace(pixels.length);
    trace("");

    trace("// pixels");
    trace(pixels);
    trace("");
}

var src: BitmapData = createSource();
printPixels(src, 0, 0, 5, 5);
printPixels(src, 5, 5, 3, 3);
printPixels(src, 0, 0, 10, 10);
printPixels(src, -1, -1, 2, 2);
