package {
	import flash.display.MovieClip;

	public class Test extends MovieClip {
		[Embed (source="test.png")]
		public static const TestBitmap:Class;

		public function Test() {
			super();
			
			trace("///var tb = new Test.TestBitmap();");
			var tb = new Test.TestBitmap();

			trace("///tb.bitmapData;");
			trace(tb.bitmapData);

			trace("///tb.pixelSnapping;");
			trace(tb.pixelSnapping);

			trace("///tb.smoothing;");
			trace(tb.smoothing);
		}
	}
}