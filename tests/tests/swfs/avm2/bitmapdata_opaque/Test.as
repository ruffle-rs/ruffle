package {
	import flash.display.Stage;
	import flash.display.BitmapData;
	import flash.display.Sprite;
	import flash.display.Bitmap;

	public class Test {
		public function Test(stage:Stage) {
			var data = new BitmapData(200, 200, false, 0);
			var rect = new Sprite();
			rect.graphics.lineStyle(2, 0xFF0000);
			rect.graphics.drawRect(10, 10, 180, 180);
			data.draw(rect);
			
			// We will render this for a single frame in Ruffle.
			// It should show a red rectangle against a black background.
			var bitmap = new Bitmap(data);
			stage.addChild(bitmap);
		}
	}
}