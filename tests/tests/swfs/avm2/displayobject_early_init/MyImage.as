package  {
	
	import flash.display.BitmapData;
	
	
	public class MyImage extends BitmapData {
		
		
		public function MyImage(width: int, height: int) {
			trace("MyImage before super(): width=" + width + " height=" + height);
			try {
				this.width;
			} catch (e) {
				trace("Caught error: " + e);
			}
			super(width, height)
			trace("MyImage after super(): this.width=" + this.width + " this.height=" + this.height + " pixel: " + this.getPixel(0, 0));
		}
	}
	
}
