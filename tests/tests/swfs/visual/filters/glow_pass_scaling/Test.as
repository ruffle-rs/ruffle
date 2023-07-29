package  {
	
import flash.display.Bitmap;
import flash.geom.Point;
import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.display.MovieClip;
import flash.display.Shape;
import flash.filters.GlowFilter;

		
public class Test extends MovieClip {
	
	private static const BLURSIZE = 7;
	
	function make_square() : Shape {
		var rectangle:Shape = new Shape;
		rectangle.graphics.beginFill(0xFFFFFF);
		rectangle.graphics.drawRect(20, 20, 40, 40);
		rectangle.graphics.endFill();
		
		return rectangle;
	}
	
	function row_1() {
		
		for (var quality = 0; quality < 10; quality += 1) {
			var filter = new GlowFilter(0xFF0000, 1.0, BLURSIZE, 1, 100, quality);
			
			var rect = make_square();
			
			rect.y = 10;
			rect.x = quality * 100 + 10;
			rect.filters = [filter]; // applied only once
			addChild(rect);
		}
	}
	
	
	function row_2() {
		
		for (var applications = 0; applications < 10; applications += 1) {
			var filter = new GlowFilter(0xFF0000, 1.0, BLURSIZE, 1, 100, 1);
			
			var filters = [];
			
			for (var i = 0; i < applications; ++i)
				filters.push(filter);
			
			var rect = make_square();
			
			rect.y = 110;
			rect.x = applications * 100 + 10;
			rect.filters = filters;
			addChild(rect);
		}
	}
	
	public function Test() {
		row_1();
		row_2();
	}
}

}