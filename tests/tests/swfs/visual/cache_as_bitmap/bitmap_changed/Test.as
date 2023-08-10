package  {
	
	import flash.display.MovieClip;
	import flash.display.Bitmap;
	import flash.display.BitmapData;
	import flash.events.Event;
	import flash.geom.Point;
	import flash.filters.ColorMatrixFilter;
	import flash.system.fscommand;
	
	
	public class Test extends MovieClip {
		var bitmap: Bitmap;
		var bmd: BitmapData;
		var frames: uint = 0;
		
		public function Test() {
			bmd = new BitmapData(120, 120, false, 0xFF0000);
			bitmap = new Bitmap(bmd);
			bitmap.cacheAsBitmap = true;
			addChild(bitmap);
			addEventListener(Event.ENTER_FRAME, this.onEnterFrame);
			fscommand("captureImage", "initial");
		}
		
		function onEnterFrame(event: Event) {
			frames++;
			if (frames == 1) {
				bmd.fillRect(bmd.rect, 0x00FF00);
				fscommand("captureImage", "fillrect");
			} else if (frames == 2) {
				var matrix:Array = new Array();
				matrix = matrix.concat([0, 0, 0, 0, 0]); // red
				matrix = matrix.concat([0, 0, 0, 0, 0]); // green
				matrix = matrix.concat([0, 1, 0, 0, 0]); // blue
				matrix = matrix.concat([0, 0, 0, 1, 0]); // alpha
				bmd.applyFilter(bmd, bmd.rect, new Point(0, 0), new ColorMatrixFilter(matrix));
				fscommand("captureImage", "applyfilter");
			}
		}
	}
	
}
