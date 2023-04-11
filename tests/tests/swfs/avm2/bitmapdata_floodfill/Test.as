package  {
	
	import flash.display.MovieClip;
	import flash.display.BitmapData;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var bmd = new BitmapData(3, 3, 0);
			test(bmd, 1, 1, 0xFFFFFF);
			test(bmd, 0, 0, 0);
			test(bmd, 2, 2, 0);
		}
		
		function test(bmd: BitmapData, x: int, y: int, color: uint) {
			trace("// bmd.floodFill(" + x + ", " + y + ", 0x" + color.toString(16) + ")");
			bmd.floodFill(x, y, color);
			trace("");
			
			dumpBmd(bmd);
			trace("");
		}
		
		function dumpBmd(bmd: BitmapData) {
			for (var x = 0; x < bmd.width; x++) {
				for (var y = 0; y < bmd.height; y++) {
					trace("// bmd.getPixel(" + x + ", " + y + ") == 0x" + bmd.getPixel(x, y).toString(16));
				}
			}
		}
	}
	
}
