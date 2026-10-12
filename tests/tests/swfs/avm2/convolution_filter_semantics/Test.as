package {
	import flash.display.BitmapData;
	import flash.display.DisplayObjectContainer;
	import flash.display.MovieClip;
	import flash.filters.ConvolutionFilter;
	import flash.geom.Point;
	import flash.geom.Rectangle;

	/**
	 * ConvolutionFilter on opaque bitmaps, pinned against Flash Player 32.
	 *
	 * output.txt was produced by running this class in flashplayer_32_sa_debug.exe.
	 * Opaque sources are the case that matters in practice: `BitmapData` created
	 * with `transparent = false` is what net.flashfilter's E01_raindrop water
	 * simulation filters every frame, and premultiplied and straight alpha
	 * coincide there, so this must match Flash exactly.
	 */
	public class Test extends MovieClip {
		public function Test() {
			Test.run(this);
		}

		public static function run(root: DisplayObjectContainer): void {
			var rect: Rectangle = new Rectangle(0, 0, 8, 4);
			var zero: Point = new Point(0, 0);

			// The 3x3 box blur the water simulation applies every frame
			var box: ConvolutionFilter = new ConvolutionFilter(3, 3, [1, 1, 1, 1, 1, 1, 1, 1, 1], 9, 0, true, true, 128);

			var flat: BitmapData = new BitmapData(8, 4, false, 0);
			flat.setPixel32(0, 0, 0xFF000000);
			flat.setPixel32(1, 0, 0xFF808080);
			flat.setPixel32(2, 0, 0xFFFFFFFF);
			var dst: BitmapData = new BitmapData(8, 4, false, 0xFF000000);
			dst.applyFilter(flat, rect, zero, box);
			trace("box3x3: " + row(dst));

			// A zero divisor is treated as 1, saturating every channel
			var zeroDiv: ConvolutionFilter = new ConvolutionFilter(3, 3, [1, 1, 1, 1, 1, 1, 1, 1, 1], 0, 0, true, true, 128);
			var dst2: BitmapData = new BitmapData(8, 4, false, 0xFF000000);
			dst2.applyFilter(flat, rect, zero, zeroDiv);
			trace("zero-divisor: " + row(dst2));

			// bias is added after the division, to every channel
			var biased: ConvolutionFilter = new ConvolutionFilter(3, 3, [1, 1, 1, 1, 1, 1, 1, 1, 1], 9, 40, true, false, 0);
			var dst3: BitmapData = new BitmapData(8, 4, false, 0xFF000000);
			dst3.applyFilter(flat, rect, zero, biased);
			trace("bias40: " + row(dst3));

			// An edge kernel: the flat regions go black, edges light up
			var edge: ConvolutionFilter = new ConvolutionFilter(3, 3, [0, -1, 0, -1, 4, -1, 0, -1, 0], 1, 0, true, true, 128);
			var dst4: BitmapData = new BitmapData(8, 4, false, 0xFF000000);
			dst4.applyFilter(flat, rect, zero, edge);
			trace("edge: " + row(dst4));

			// Only the pixels inside sourceRect are written
			var id: ConvolutionFilter = new ConvolutionFilter(1, 1, [1], 1, 0);
			var dst5: BitmapData = new BitmapData(8, 4, false, 0xFF112233);
			dst5.applyFilter(flat, new Rectangle(1, 0, 2, 1), zero, id);
			trace("sourceRect: " + row(dst5));

			// Convolving a 1px line, as the water simulation's initial
			// `fillRect` drop does
			var drop: BitmapData = new BitmapData(8, 4, false, 0);
			drop.setPixel32(3, 0, 0xFFFFFFFF);
			drop.setPixel32(4, 0, 0xFFFFFFFF);
			var dst6: BitmapData = new BitmapData(8, 4, false, 0xFF000000);
			dst6.applyFilter(drop, rect, zero, box);
			trace("drop: " + row(dst6));
		}

		static function row(b: BitmapData): String {
			var s: String = "";
			for (var x: int = 0; x < 5; x++) {
				s += b.getPixel32(x, 0).toString(16) + " ";
			}
			return s;
		}
	}
}
