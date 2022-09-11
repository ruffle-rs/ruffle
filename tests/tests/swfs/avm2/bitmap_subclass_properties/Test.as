package {
	import flash.display.MovieClip;

	public class Test extends MovieClip {
		[Embed (source="test.png")]
		public static const TestBitmap:Class;

		public function Test() {
			super();
			
			trace("///var tb = new Test.TestBitmap();");
			var tb = new Test.TestBitmap();

			trace("///tb.bitmapData.width;");
			trace(tb.bitmapData.width);

			trace("///tb.bitmapData.height;");
			trace(tb.bitmapData.height);

			trace("///tb.bitmapData.getPixel(0,0);");
			trace(tb.bitmapData.getPixel(0,0));

			trace("///tb.bitmapData.getPixel(12,12);");
			trace(tb.bitmapData.getPixel(12,12));
		}
	}
}