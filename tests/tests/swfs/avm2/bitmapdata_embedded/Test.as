package {
	public class Test {
	}
}

trace("///var tbd = new TestBitmapData(128, 128);");
var tbd = new TestBitmapData(128, 128);

trace("///tbd.width;");
trace(tbd.width);

trace("///tbd.height;");
trace(tbd.height);

trace("///tbd.getPixel(0,0);");
trace(tbd.getPixel(0,0));

trace("///tbd.getPixel(12,12);");
trace(tbd.getPixel(12,12));