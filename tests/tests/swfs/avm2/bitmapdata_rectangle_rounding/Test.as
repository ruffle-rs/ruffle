package {
	import flash.display.BitmapData;
	import flash.geom.Rectangle;

	public class Test {
		public function Test() {
			var otherData = new BitmapData(5, 5, true, 0);
			otherData.fillRect(new Rectangle(1.6, 1.6, 1.8, 2), 0xFFFFFFFF);
			trace("First fillRect");
			print(otherData);
			
			otherData.fillRect(new Rectangle(0.1, 0, 1.8, 2.5), 0xFFaabbcc);
			trace("Second fillRect");
			print(otherData);
			
			var bytes = otherData.getPixels(new Rectangle(0.5, 0.5, 2.5, 2.5));
			bytes.position = 0;
			trace("Bytes len: " + bytes.length);
			var byteString = "";
			while (bytes.bytesAvailable != 0) {
				byteString += bytes.readUnsignedInt().toString(16) + " ";
			}
			trace("Bytes: " + byteString);
		}
	
		private function print(data: BitmapData) {
			for (var y = 0; y < data.height; y++) {
				var line = "";
				for (var x = 0; x < data.width; x++) {
					line += data.getPixel32(x, y).toString(16) + " "
				}
				trace(line)
			}
			trace()
		}
	}
}