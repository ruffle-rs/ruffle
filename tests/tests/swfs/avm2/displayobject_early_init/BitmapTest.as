package  {
	
	import flash.display.BitmapData;
	
	
	// FIXME - this should extend Bitmap
	public class BitmapTest extends BitmapData {
		
		
		public function BitmapTest() {
			//trace("BitmapTest before super(): this.bitmapData.width = " + this.bitmapData.width);
			super(100, 100);
		}
	}
	
}
