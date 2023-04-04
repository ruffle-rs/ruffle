package  {
	
	import flash.display.MovieClip;
	import flash.display.BitmapData;
	import flash.display.Bitmap;
	import flash.geom.Rectangle;
	import flash.events.Event;
	
	
	public class Test extends MovieClip {
		static const WIDTH: uint = 30;
		static const HEIGHT: uint = 30;

		private var bmd: BitmapData;
		private var testNumber: uint = 0;
		
		public function Test() {
			// We reuse the same BMD to see if any sync errors will compound
			bmd = new BitmapData(WIDTH, HEIGHT, false, 0);

			var bitmap = new Bitmap(bmd);
			bitmap.x = Math.floor((stage.stageWidth / 2) - (WIDTH / 2));
			bitmap.y = Math.floor((stage.stageHeight / 2) - (HEIGHT / 2));
			addChild(bitmap);

            addEventListener(Event.ENTER_FRAME, onFrame);
		}

		function onFrame(event: Event) {
			if (testNumber == 0) {
				bmd.fillRect(new Rectangle(5, 5, 5, 5), 0xFFFFFF);
			} else if (testNumber == 1) {
				bmd.setPixel(1, 1, 0xFF0000);
			} else if (testNumber == 2) {
				bmd.setPixel(6, 6, 0x0000FF);
				bmd.setPixel(8, 8, 0x0000FF);
				bmd.setPixel(10, 10, 0x0000FF);
			} else if (testNumber == 3) {
				bmd.setPixel32(12, 12, 0xFF0000);
				bmd.setPixel32(14, 14, 0xFF0000);
				bmd.setPixel32(16, 16, 0xFF0000);
			} else if (testNumber == 4) {
				removeEventListener(Event.ENTER_FRAME, onFrame);
			}

			testNumber++;
		}
	}
	
}
