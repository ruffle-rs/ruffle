package {
	import flash.display.BitmapData;
	import flash.display.Bitmap;

	public class Test {
		public function Test() {
			var sourceData = new BitmapData(4, 4, true, 0x44aaaaff);
			
			var alphaData = new BitmapData(4, 4, true, 0xFF224466);
			trace("alphaData before alpha: " + alphaData.getPixel32(0, 0).toString(16));
			alphaData.draw(new Bitmap(sourceData), null, null, "alpha");
			trace("alphaData after alpha: " + alphaData.getPixel32(0, 0).toString(16));
			
			var eraseData = new BitmapData(4, 4, true, 0xFF202060);
			trace("eraseData before erase: " + eraseData.getPixel32(0, 0).toString(16));
			eraseData.draw(new Bitmap(sourceData), null, null, "erase");
			trace("eraseData after erase: " + eraseData.getPixel32(0, 0).toString(16));
	
			var dummyAlphaData = new BitmapData(4, 4, true, 0xFF224466);
			trace("dummyAlphaData before alpha: " + dummyAlphaData.getPixel32(0, 0).toString(16));
			dummyAlphaData.draw(sourceData, null, null, "alpha");
			trace("dummyAlphaData after alpha: " + dummyAlphaData.getPixel32(0, 0).toString(16));
		
			var dummyEraseData = new BitmapData(4, 4, true, 0xFFaabbcc);
			trace("dummyEraseData before erase: " + dummyEraseData.getPixel32(0, 0).toString(16));
			dummyEraseData.draw(sourceData, null, null, "alpha");
			trace("dummyEraseData after erase: " + dummyEraseData.getPixel32(0, 0).toString(16));
			
		}
	}
}