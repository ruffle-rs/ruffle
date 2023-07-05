package  {
	
	import flash.display.MovieClip;
	import flash.display.Bitmap;
	import flash.display.PixelSnapping;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			addTest(PixelSnapping.NEVER, 1.0);
			addTest(PixelSnapping.ALWAYS, 1.0);
			addTest(PixelSnapping.AUTO, 1.0);
			
			addTest(PixelSnapping.NEVER, 0.99);
			addTest(PixelSnapping.ALWAYS, 0.99);
			addTest(PixelSnapping.AUTO, 0.99);
			
			addTest(PixelSnapping.NEVER, 1.01);
			addTest(PixelSnapping.ALWAYS, 1.01);
			addTest(PixelSnapping.AUTO, 1.01);
			
			addTest(PixelSnapping.NEVER, 2.0);
			addTest(PixelSnapping.ALWAYS, 2.0);
			addTest(PixelSnapping.AUTO, 2.0);
			
			addTest(PixelSnapping.NEVER, 2.5);
			addTest(PixelSnapping.ALWAYS, 2.5);
			addTest(PixelSnapping.AUTO, 2.5);
			
			try {
				var bitmap = new Bitmap(new TestImage());
				bitmap.pixelSnapping = "test";
			} catch (err) {
				trace(err);
			}
			
			try {
				var bitmap = new Bitmap(new TestImage(), "test");
			} catch (err) {
				trace(err);
			}
		}
		
		function addTest(snapping: String, scale: Number) {
			var bitmap = new Bitmap(new TestImage());
			bitmap.pixelSnapping = snapping;
			bitmap.x = (numChildren % 3 * 100) + 10.5;
			bitmap.y = (Math.floor(numChildren / 3) * 100) + 10.5;
			bitmap.width *= scale;
			bitmap.height *= scale;
			addChild(bitmap);
		}
	}
	
}
