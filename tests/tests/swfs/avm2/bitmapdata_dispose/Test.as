package {
	import flash.display.Bitmap;

	public class Test {
		public function Test() {
			import flash.display.BitmapData;

			var data = new BitmapData(5, 6);
			var bitmapBefore = new Bitmap(data);
			trace("bitmapBefore: width=" + bitmapBefore.width + " height=" + bitmapBefore.height);
			trace("//data.dispose()");
			data.dispose();
			
			trace("bitmapBefore: width=" + bitmapBefore.width + " height=" + bitmapBefore.height);
			var bitmapAfter = new Bitmap(data);
			trace("bitmapAfter: width=" + bitmapAfter.width + " height=" + bitmapAfter.height);
			
			trace("//bitmapBefore.bitmapData = data");
			bitmapBefore.bitmapData = data;
			trace("bitmapBefore: width=" + bitmapBefore.width + " height=" + bitmapBefore.height);
			
			try {
				trace("ERROR: Read width " + data.width);
			} catch (e) {
				trace("Caught error: ", e);
			}			
		}
	}
}