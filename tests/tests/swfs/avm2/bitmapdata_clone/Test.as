package {
	public class Test {
	}
}

import flash.display.BitmapData;

trace("///var tbd = new TestBitmapData(128, 128).clone();");
var tbd = new TestBitmapData(128, 128).clone();

trace("///tbd.width;");
trace(tbd.width);

trace("///tbd.height;");
trace(tbd.height);

trace("///tbd.getPixel(0,0);");
trace(tbd.getPixel(0,0));

trace("///tbd.getPixel(12,12);");
trace(tbd.getPixel(12,12));

trace("///tbd instanceof BitmapData");
trace(tbd instanceof BitmapData);

trace("///tbd instanceof TestBitmapData");
trace(tbd instanceof TestBitmapData);