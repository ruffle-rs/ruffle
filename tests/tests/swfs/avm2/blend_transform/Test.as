package {
	import flash.display.MovieClip;
	import flash.display.BitmapData;
	import flash.display.Bitmap;
	import flash.display.BlendMode;
	import flash.geom.Rectangle;
	import flash.geom.ColorTransform;
	
	public class Test {
		public static function test(main:MovieClip) {
			var data = new BitmapData(200, 200, true, 0xddFF0000);
			var bitmap = new Bitmap(data);
			
			var secondData = new BitmapData(50, 50, true);
			secondData.fillRect(new Rectangle(0, 0, 50, 50), 0xaaeeeeee);
			data.draw(secondData, null, null, BlendMode.DIFFERENCE);
			
			trace("Pixel (1, 1): " + data.getPixel(1, 1).toString(16));
			
			main.addChild(bitmap);
			testSubtract(main);
		}
	
		static function testSubtract(main:MovieClip) {
			var data = new BitmapData(200, 200, true, 0xFFaaaaaa);
			var bitmap = new Bitmap(data);
			
			var secondData = new BitmapData(50, 50, true);
			secondData.fillRect(new Rectangle(0, 0, 50, 50), 0xFF000000);
			// This should blend against the existinc contents, subtracting '0' and leaving it unchanged
			data.draw(secondData, null, null, BlendMode.SUBTRACT);
			
			bitmap.y = 220;
			main.addChild(bitmap);
		}
	}
}