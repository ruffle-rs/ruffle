package {
	import flash.display.MovieClip;

	public class Test {
		public function Test(main:MovieClip) {
			applyBlur(main);
			applyColorMatrix(main);
		}
	}
}

import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.display.Bitmap;
import flash.filters.BlurFilter;
import flash.filters.ColorMatrixFilter;
import flash.geom.Point;
import flash.display.MovieClip;

function applyBlur(movie: MovieClip) {
	var sourceData = new BitmapData(200, 200, true, 0xFFFF0000);
	sourceData.fillRect(new Rectangle(50, 50, 50, 50), 0xFF000000);

	var dest = new BitmapData(200, 200, true, 0xFF00FF00);
	dest.applyFilter(sourceData, new Rectangle(70, 50, 100, 100), new Point(10, 30), new BlurFilter());

	var bitmap = new Bitmap(dest);
	bitmap.x = 20;
	bitmap.y = 20;

	var sourceBitmap = new Bitmap(sourceData);
	sourceBitmap.x = 300;
	sourceBitmap.y = 20;

	movie.addChild(bitmap);
	movie.addChild(sourceBitmap);
}

function applyColorMatrix(movie: MovieClip) {
	var sourceData = new BitmapData(200, 200, true, 0xFFFF0000);
	sourceData.fillRect(new Rectangle(50, 50, 50, 50), 0xFF000000);

	var dest = new BitmapData(200, 200, true, 0xFF00FF00);
	var matrix = [
				0, 1, 0, 0, 0,
				0, 0, 1, 0, 0,
				0, 0, 0, 1, 0,
				1, 0, 0, 0, 0,
			];
	dest.applyFilter(sourceData, new Rectangle(70, 50, 100, 100), new Point(10, 30), new ColorMatrixFilter(matrix));

	var bitmap = new Bitmap(dest);
	bitmap.x = 20;
	bitmap.y = 250;

	var sourceBitmap = new Bitmap(sourceData);
	sourceBitmap.x = 300;
	sourceBitmap.y = 250;

	movie.addChild(bitmap);
	movie.addChild(sourceBitmap);	
}