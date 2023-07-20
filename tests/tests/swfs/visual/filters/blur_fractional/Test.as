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
				var image  : BitmapData = new Image();
				
				var filter = new BlurFilter(row + col * 0.1, 1, 1);
				image.applyFilter(image, new Rectangle(0, 0, 80, 80), new Point(0, 0), filter);
				
				var bm = new Bitmap(image);
				bm.smoothing = false;
				bm.y = row * 80;
				bm.x = col * 80;
				addChild(bm);
			}
		}
	}
}

}