import flash.display.BitmapData;
import flash.geom.Point;

// Bitmap layout (5x5, all transparent except):
//   (2,2) alpha=0x00  fully transparent
//   (2,4) alpha=0xFF  fully opaque
var bmd = new BitmapData(5, 5, true, 0x00000000);
bmd.setPixel32(2, 4, 0xFFFFFFFF);

var origin = new Point(0, 0);

trace("threshold=0, transparent pixel (alpha=0):");
trace(bmd.hitTest(origin, 0, new Point(2, 2)));

trace("threshold=0, opaque pixel (alpha=0xFF):");
trace(bmd.hitTest(origin, 0, new Point(2, 4)));

trace("threshold=0, out-of-bounds point:");
trace(bmd.hitTest(origin, 0, new Point(10, 10)));
