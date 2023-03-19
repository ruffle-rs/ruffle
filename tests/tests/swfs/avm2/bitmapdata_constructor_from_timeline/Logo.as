package  {
	
	import flash.display.BitmapData;
	
	
	public class Logo extends BitmapData {
		
		
		public function Logo(width: Number, height: Number) {
			super(width, height);
			trace("Logo constructor called with: " + width + ", " + height);
		}
	}
	
}
