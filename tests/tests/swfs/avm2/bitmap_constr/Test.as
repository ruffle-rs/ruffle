package {
	public class Test {
	}
}

import flash.display.Bitmap;

trace("///var b1 = new Bitmap();");
var b1 = new Bitmap();

trace("///b1.bitmapData;");
trace(b1.bitmapData);

trace("///b1.pixelSnapping;");
trace(b1.pixelSnapping);

trace("///b1.smoothing;");
trace(b1.smoothing);

trace("///var tbd = new TestBitmapData();");
var tbd = new TestBitmapData();

trace("///var b2 = new Bitmap(tbd);");
var b2 = new Bitmap(tbd);

trace("///b2.bitmapData;");
trace(b2.bitmapData);

trace("///b2.pixelSnapping;");
trace(b2.pixelSnapping);

trace("///b2.smoothing;");
trace(b2.smoothing);

trace("///b2.bitmapData === tbd;");
trace(b2.bitmapData === tbd);