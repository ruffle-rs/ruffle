package  {

import flash.display.MovieClip;
import flash.display.BitmapData;
import flash.display.Bitmap;
import flash.display.BitmapDataChannel;
import flash.filters.DisplacementMapFilter;
import flash.filters.DisplacementMapFilterMode;
import flash.geom.Matrix;
import flash.display.GradientType;
import flash.display.SpreadMethod;
import flash.display.Sprite;
import flash.geom.Point;


public class Test extends MovieClip {


	public function Test() {
		var bmd = createBitmapData();
		var test;

		test = new TestImage();
		test.filters = [new DisplacementMapFilter()];
		test.x = 250;
		addChild(test);

		test = new TestImage();
		test.filters = [new DisplacementMapFilter(bmd, new Point(0, 0), BitmapDataChannel.RED, BitmapDataChannel.BLUE, 30, -30, DisplacementMapFilterMode.CLAMP)];
		test.y = 250;
		addChild(test);

		test = new TestImage();
		test.filters = [new DisplacementMapFilter(bmd, new Point(0, 0), BitmapDataChannel.BLUE, BitmapDataChannel.GREEN, -30, 30, DisplacementMapFilterMode.WRAP)];
		test.y = 250;
		test.x = 250;
		addChild(test);

		test = new TestImage();
		test.filters = [new DisplacementMapFilter(bmd, new Point(50, 20), BitmapDataChannel.RED, BitmapDataChannel.RED, 15, -15, DisplacementMapFilterMode.COLOR, 0xFF0000, 0.5)];
		test.y = 500;
		addChild(test);

		test = new TestImage();
		test.filters = [new DisplacementMapFilter(bmd, new Point(-10, -10), BitmapDataChannel.BLUE, BitmapDataChannel.GREEN, 50, 50, DisplacementMapFilterMode.IGNORE)];
		test.y = 500;
		test.x = 250;
		addChild(test);

		// Setting componentX or componentY to 0 is undocumented, but works (as no-op in that direction):

		test = new TestImage();
		test.filters = [new DisplacementMapFilter(bmd, new Point(0, 0), 0, BitmapDataChannel.RED, -30, 30, DisplacementMapFilterMode.CLAMP)];
		test.y = 750;
		addChild(test);

		test = new TestImage();
		test.filters = [new DisplacementMapFilter(bmd, new Point(0, 0), BitmapDataChannel.RED, 0, -30, 30, DisplacementMapFilterMode.WRAP)];
		test.y = 750;
		test.x = 250;
		addChild(test);
	}

	private function createBitmapData():BitmapData {
		var matrix:Matrix = new Matrix();
		var gradient = new Sprite();
		matrix.createGradientBox(200, 200);
		gradient.graphics.beginGradientFill(GradientType.RADIAL,
			[0xFF0000, 0x0000FF],
			[100, 100],
			[55, 200],
			matrix,
			SpreadMethod.PAD);
		gradient.graphics.drawRect(0, 0, 200, 200);
		var bitmapData:BitmapData = new BitmapData(200, 200, true, 0xAABBCC);
		bitmapData.draw(gradient, new Matrix());
		var bitmap:Bitmap = new Bitmap(bitmapData);
		addChild(bitmap);
		return bitmapData;
	}
}
}
