package {
	import flash.display.DisplayObjectContainer;
	import flash.display.BitmapData;
	import flash.geom.Rectangle;
	import flash.utils.ByteArray;

	public class Test {
		static var WIDTH: uint = 10;
		static var HEIGHT: uint = 10;

		public function Test() {
			var src: BitmapData = createSource();
			printPixels(src, 0, 0, 5, 5);
			printPixels(src, 5, 5, 3, 3);
			printPixels(src, 0, 0, 10, 10);
			printPixels(src, -1, -1, 2, 2);
		}

		static function createSource(): BitmapData {
			var src: BitmapData = new BitmapData(WIDTH, HEIGHT, true, 0x00000000);
			src.noise(0);
			return src;
		}

		static function printPixels(src: BitmapData, x: int, y: int, width: uint, height: uint): void {
			var rect: Rectangle = new Rectangle(x, y, width, height);
			var pixels = new ByteArray();
			pixels.writeByte(42);
			pixels.writeByte(5);
			pixels.position = 100;
			pixels.writeByte(255);
			pixels.position = 1;
			src.copyPixelsToByteArray(rect, pixels);

			trace("/// copyPixelsToByteArray(new Rectangle(" + x + ", " + y + ", " + width + ", " + height + "))");

			trace("// pixels.length");
			trace(pixels.length);
			trace("");

			trace("// pixels.position");
			trace(pixels.position);
			trace("");

			trace("// pixels");
			var result = [];
			pixels.position = 0;
			for (var i = 0; i < pixels.length; i++) {
				result.push(pixels.readUnsignedByte());
			}
			trace(result);
			trace("");
		}
	}
}
