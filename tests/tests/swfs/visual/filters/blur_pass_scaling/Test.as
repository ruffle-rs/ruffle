package  {
	
import flash.display.Bitmap;
import flash.geom.Point;
import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.display.MovieClip;
import flash.filters.BlurFilter;

		
public class Test extends MovieClip {
	
	private static const BLURSIZE = 7;

	function row_1() {
		
		for (var quality = 0; quality < 10; quality += 1) {
			var image : BitmapData = new Image();
			
			var filter = new BlurFilter(BLURSIZE, 1, quality);
			image.applyFilter(image, new Rectangle(0, 0, 80, 80), new Point(0, 0), filter); // applied only once
			
			var bm = new Bitmap(image);
			bm.smoothing = false;
			bm.y = 10;
			bm.x = quality * 100 + 10;
			addChild(bm);
		}
	}
	
	function row_2() {
		
		for (var applications = 0; applications < 10; applications += 1) {
			var image : BitmapData = new Image();
			
			var filter = new BlurFilter(BLURSIZE, 1, 1); // quality is always 1
			for (var i = 0; i < applications; ++i)
				image.applyFilter(image, new Rectangle(0, 0, 80, 80), new Point(0, 0), filter);
			
			var bm = new Bitmap(image);
			bm.smoothing = false;
			bm.y = 110;
			bm.x = i * 100 + 10;
			addChild(bm);
		}
	}
	
	
	function row_3() {
		
		for (var quality = 0; quality < 10; quality += 1) {
			var image : BitmapData = new Image();
			
			var filter = new BlurFilter(BLURSIZE, 1, quality);
			
			var bm = new Bitmap(image);
			bm.smoothing = false;
			bm.y = 210;
			bm.x = quality * 100 + 10;
			bm.filters = [filter]; // applied only once
			addChild(bm);
		}
	}
	
	
	function row_4() {
		
		for (var applications = 0; applications < 10; applications += 1) {
			var image : BitmapData = new Image();
			
			var filter = new BlurFilter(BLURSIZE, 1, 1); // quality is always 1
			
			var filters = [];
			
			for (var i = 0; i < applications; ++i)
				filters.push(filter);
				
			var bm = new Bitmap(image);
			bm.smoothing = false;
			bm.y = 310;
			bm.x = i * 100 + 10;
			bm.filters = filters;
			addChild(bm);
		}
	}
	
	public function Test() {
		
		row_1();
		row_2();
		row_3();
		row_4();
		
		
		
	}
}

}