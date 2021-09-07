package {
	public class Test {
	}
}

import flash.display.Bitmap;

trace("///var b1 = new Bitmap();");
var b1 = new Bitmap();

trace("///b1.width;");
trace(b1.width);

trace("///b1.height;");
trace(b1.height);

trace("///var tbd = new TestBitmapData();");
var tbd = new TestBitmapData();

trace("///var b2 = new Bitmap(tbd);");
var b2 = new Bitmap(tbd);

trace("///b2.width;");
trace(b2.width);

trace("///b2.height;");
trace(b2.height);

trace("///b2.bitmapData = null;");
trace(b2.bitmapData = null);

trace("///b2.width;");
trace(b2.width);

trace("///b2.height;");
trace(b2.height);

trace("///b2.bitmapData = tbd;");
trace(b2.bitmapData = tbd);

trace("///b2.width;");
trace(b2.width);

trace("///b2.height;");
trace(b2.height);