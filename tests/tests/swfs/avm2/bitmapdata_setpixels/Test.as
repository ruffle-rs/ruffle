package  {
	
	import flash.display.MovieClip;
	import flash.display.BitmapData;
	import flash.geom.Rectangle;
	import flash.utils.ByteArray;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var ba: ByteArray = makeByteArray();
			var bmd: BitmapData;

			test(false, 0, ba, new Rectangle(0, 0, 3, 3));
			test(true, 0, ba, new Rectangle(0, 0, 3, 3));
			test(true, 10, ba, new Rectangle(0, 0, 3, 3));
			test(true, 0, ba, new Rectangle(-1, -1, 3, 3));
			test(true, 0, ba, new Rectangle(-1, -1, 4, 4));
			test(true, 0, ba, new Rectangle(-5, -4, 3, 2));
			test(true, 0, ba, new Rectangle(1, 1, 1, 1));
			test(true, 0, ba, new Rectangle(1, 1, 10, 10));
			test(true, 30, ba, new Rectangle(1, 1, 1, 1));
		}

		function test(transparency: Boolean, position: uint, ba: ByteArray, rect: Rectangle) {
			trace("/// new ByteArray(3, 3, " + transparency + ", 0).setPixels(new Rectangle(" + rect.x + ", " + rect.y + ", " + rect.width + ", " + rect.height + "), ba)");
			var bmd = new BitmapData(3, 3, transparency, 0);
			ba.position = position;
			try {
				bmd.setPixels(rect, ba);
			} catch (error: Error) {
				trace(error);
			}
			dump(bmd, ba);
			trace("");
		}

		function dump(bmd: BitmapData, ba: ByteArray) {
			for (var x = 0; x < bmd.width; x++) {
				for (var y = 0; y < bmd.height; y++) {
					trace("// bmd.getPixel32(" + x + ", " + y + ")");
					trace(bmd.getPixel32(x, y));
					trace("");
				}
			}
			trace("// ba.position");
			trace(ba.position);
			trace("");
		}

		function makeByteArray(): ByteArray {
			var result = new ByteArray();
			var len = 4 * 4 * 4;

			for (var i = 0; i < len; i++) {
				result.writeByte((i / len) * 255);
			}

			return result;
		}
	}
	
}
