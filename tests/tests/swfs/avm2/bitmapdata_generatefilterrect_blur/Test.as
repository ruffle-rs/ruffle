package  {
	
import flash.display.Bitmap;
import flash.geom.Point;
import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.display.MovieClip;
import flash.filters.BlurFilter;

public class Test extends MovieClip {

	public function Test() {
		
		for (var col = 0; col < 10; col += 1) {
			for (var row = 0; row < 10; row += 1) {
				var image : BitmapData = new Image();
				
				var filter = new BlurFilter(row, 1, col);
				image.applyFilter(image, new Rectangle(0, 0, 80, 80), new Point(0, 0), filter);
				
				var filterRect1 = image.generateFilterRect(new Rectangle(40, 40, 1, 1), filter);
				image.setPixel(filterRect1.x-1, 0, 0xFF0000);
				image.setPixel(filterRect1.x + filterRect1.width, 0, 0x00FF00);
				
				var filterRect2 = image.generateFilterRect(new Rectangle(35, 66, 11, 11), filter);
				image.setPixel(filterRect2.x-1, 77, 0xFF0000);
				image.setPixel(filterRect2.x + filterRect2.width, 77, 0x00FF00);
				
				var bm = new Bitmap(image);
				bm.smoothing = false;
				bm.y = row * 80;
				bm.x = col * 80;
				addChild(bm);
			}
		}
		
		trace("blur size,quality,rect growth");
		
		var bmd: BitmapData = new BitmapData(1, 1);

		for (var quality = 0; quality <= 15; quality += 1) {
			for (var size = 0; size <= 100; size += 1) {
				
				var filter = new BlurFilter(size, 1, quality);
				var filterRect = image.generateFilterRect(new Rectangle(0, 0, 1, 1), filter);
				
				trace(size + "," + quality + "," + (filterRect.width - 1) / 2);
			}
		}
	}
}

}