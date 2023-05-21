package {
	import flash.display.MovieClip;
	import flash.display.BitmapData;
	import flash.geom.Rectangle;
	import flash.display.Bitmap;

	public class Test {
		public function Test(main: MovieClip) {
			var background = new BitmapData(100, 100, true, 0x0000FF00);

			var data = new BitmapData(100, 100, true, 0);
			data.fillRect(new Rectangle(0, 0, 30, 30), 0xaabb0000);

			background.draw(data, null, null, "multiply");
			main.addChild(new Bitmap(background));
		}
	}
}