package {
	public class Test {
	}
}

import flash.display.BitmapData;

trace("///var bd = new BitmapData(128, 128, true, 0xCAFEBABE);");
var bd = new BitmapData(128, 128, true, 0xCAFEBABE);

trace("///bd.width;");
trace(bd.width);

trace("///bd.height;");
trace(bd.height);

trace("///bd.transparent;");
trace(bd.transparent);

trace("///bd.getPixel(0,0);");
trace(bd.getPixel(0,0));

trace("///bd = new BitmapData(128, 128, false, 0xCAFEBABE);");
bd = new BitmapData(128, 128, false, 0xCAFEBABE);

trace("///bd.width;");
trace(bd.width);

trace("///bd.height;");
trace(bd.height);

trace("///bd.transparent;");
trace(bd.transparent);

trace("///bd.getPixel(0,0);");
trace(bd.getPixel(0,0));