package {
	public class Test {
		public function Test() {
			import flash.display.BitmapData;

			var data = new BitmapData(5, 5);
			data.dispose();
			try {
				trace("ERROR: Read width " + data.width);
			} catch (e) {
				trace("Caught error: ", e);
			}			
		}
	}
}