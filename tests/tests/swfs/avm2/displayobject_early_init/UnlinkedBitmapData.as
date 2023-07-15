package {
	import flash.display.BitmapData;

	public class UnlinkedBitmapData extends BitmapData {
		public function UnlinkedBitmapData() {
			trace("UnlinkedBitmapData before super()");
			try {
				this.width;
			} catch (e) {
				trace("Caught error: " + e);
			}
			super(100, 100);
			trace("UnlinkedBitmapData after super(): this.width=" + this.width);
		} 
	}
}