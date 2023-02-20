package {
	import flash.display.MovieClip;
	import flash.display.Shape;
	import flash.display.Bitmap;
	import flash.display.BitmapData;
	
	public class Test {
		public static function test(main:MovieClip) {
			var data = new BitmapData(100, 100, false, 0x000000ff);
			
			var redCircle = new Shape();
			redCircle.graphics.beginFill(0xFFaa0000);
			redCircle.graphics.drawCircle(40, 40, 25);
			redCircle.graphics.endFill();
			
			data.draw(redCircle);
			data.scroll(-20, 0);

			var greenCircle = new Shape();
			greenCircle.graphics.beginFill(0xFF00aa00);
			greenCircle.graphics.drawCircle(40, 40, 25);
			greenCircle.graphics.endFill();
			
			data.draw(greenCircle);
			
			main.addChild(new Bitmap(data));
		}
	}
}