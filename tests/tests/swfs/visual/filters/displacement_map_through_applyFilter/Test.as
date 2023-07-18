package  {

import flash.display.Bitmap;
import flash.filters.DisplacementMapFilter;
import flash.geom.Point;
import flash.display.BitmapDataChannel;
import flash.display.BitmapData;
import flash.geom.Rectangle;
import flash.display.MovieClip;

public class Test extends MovieClip {
	public function Test() {
		var displacement1  : BitmapData = new Displacement1();
		var source1 : BitmapData = new Source1();

		var filter1 = new DisplacementMapFilter(displacement1, new Point(0,0), BitmapDataChannel.RED, BitmapDataChannel.GREEN, 200, 200);

		source1.applyFilter(source1, new Rectangle(0, 0, 175, 175), new Point(0, 0), filter1);

		var bm1 = new Bitmap(source1);
		bm1.smoothing = false;

		addChild(bm1);

		for (var strength = 200; strength < 300; strength += 10) {
			var displacement2  : BitmapData = new Displacement2();
			var source2 : BitmapData = new Source2();

			var filter2 = new DisplacementMapFilter(displacement2, new Point(0,0), BitmapDataChannel.RED, BitmapDataChannel.GREEN, strength, strength);

			source2.applyFilter(source2, new Rectangle(0, 0, 175, 20), new Point(0, 0), filter2);

			var bm2 = new Bitmap(source2);
			bm2.smoothing = false;
			bm2.y = (strength - 200) * 2;
			bm2.x = 180;
			addChild(bm2);
		}
	}
}

}