package {
	import flash.display.MovieClip;
	import flash.display.BitmapData;
	import flash.geom.Point;

	public class Test extends MovieClip {

		public function Test() {
			// Bitmap layout (5x5, all transparent except):
			//   (2,2) alpha=0x00  fully transparent
			//   (2,3) alpha=0xA0  semi-transparent (160)
			//   (2,4) alpha=0xFF  fully opaque
			var bmd:BitmapData = new BitmapData(5, 5, true, 0x00000000);
			bmd.setPixel32(2, 3, 0xA0FFFFFF);
			bmd.setPixel32(2, 4, 0xFFFFFFFF);

			var origin:Point = new Point(0, 0);

			// --- hit_test_point: threshold = 0 ---
			// Transparent pixel: answers whether threshold=0 treats alpha=0 as a hit
			trace("threshold=0, transparent pixel (alpha=0):");
			trace(bmd.hitTest(origin, 0, new Point(2, 2)));

			// Opaque pixel: must still hit with threshold=0
			trace("threshold=0, opaque pixel (alpha=0xFF):");
			trace(bmd.hitTest(origin, 0, new Point(2, 4)));

			// Out-of-bounds: must be false regardless
			trace("threshold=0, out-of-bounds point:");
			trace(bmd.hitTest(origin, 0, new Point(10, 10)));

			// --- hit_test_point: threshold = N, pixel alpha = N ---
			// Distinguishes > vs >= : alpha=0xA0=160, threshold=0xA0=160
			// If Flash uses >=: true. If Flash uses >: false.
			trace("threshold=0xA0, pixel alpha=0xA0 (boundary):");
			trace(bmd.hitTest(origin, 0xA0, new Point(2, 3)));

			// One below boundary: alpha=0xA0, threshold=0xA1 → must be false either way
			trace("threshold=0xA1, pixel alpha=0xA0 (just below):");
			trace(bmd.hitTest(origin, 0xA1, new Point(2, 3)));

			// One above boundary: alpha=0xA0, threshold=0x9F → must be true either way
			trace("threshold=0x9F, pixel alpha=0xA0 (just above):");
			trace(bmd.hitTest(origin, 0x9F, new Point(2, 3)));

			// --- hit_test_bitmapdata: threshold = 0 ---
			// Two fully-transparent bmds: Flash returns true (straight alpha >= threshold, no clamp)
			var trans1:BitmapData = new BitmapData(3, 3, true, 0x00000000);
			var trans2:BitmapData = new BitmapData(3, 3, true, 0x00000000);
			trace("bmd-vs-bmd threshold=0, both transparent:");
			trace(trans1.hitTest(origin, 0, trans2, origin, 0));

			// Two opaque bmds: must collide
			var opaque1:BitmapData = new BitmapData(3, 3, true, 0xFFFFFFFF);
			var opaque2:BitmapData = new BitmapData(3, 3, true, 0xFFFFFFFF);
			trace("bmd-vs-bmd threshold=0, both opaque:");
			trace(opaque1.hitTest(origin, 0, opaque2, origin, 0));

			// src opaque, test transparent: still true with threshold=0 (both sides satisfy alpha >= 0)
			trace("bmd-vs-bmd threshold=0, src opaque / test transparent:");
			trace(opaque1.hitTest(origin, 0, trans1, origin, 0));
		}
	}
}
