package  {
	
	import flash.display.MovieClip;
	import flash.display.Bitmap;
	import flash.display.BitmapData;
	import flash.geom.Point;
	import flash.geom.Rectangle;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var bmd: BitmapData = createImage();
			var otherBmd: BitmapData = createImage();
			var bitmap: Bitmap = new Bitmap(bmd);



			// Testing bmd against bmd, aligns both images so both points overlap and checks for any opaque overlap
			trace("/// hitTest with bmd");
			test(bmd, new Point(0, 0), 0, bmd, new Point(0, 0), 0);
			test(bmd, new Point(1, 1), 0xFF, bmd, new Point(3, 3), 0xA0);
			test(bmd, new Point(2, 1), 0xA0, bmd, new Point(1, 3), 0xA0);
			test(bmd, new Point(3, 1), 0xA0, bmd, new Point(1, 2), 0xFF);
			test(bmd, new Point(0, 0), 0xA0, bmd, new Point(1, 0), 0xFF);
			test(bmd, new Point(1, 1), 0xFF, bmd, new Point(1, 1), 0xFF);
			test(bmd, new Point(-1, -1), 0xA0, bmd, new Point(1, 1), 0xA0);
			trace("");

			trace("/// hitTest with other bmd");
			test(bmd, new Point(0, 0), 0, otherBmd, new Point(0, 0), 0);
			test(bmd, new Point(1, 1), 0xFF, otherBmd, new Point(3, 3), 0xA0);
			test(bmd, new Point(2, 1), 0xA0, otherBmd, new Point(1, 3), 0xA0);
			test(bmd, new Point(3, 1), 0xA0, otherBmd, new Point(1, 2), 0xFF);
			test(bmd, new Point(0, 0), 0xA0, otherBmd, new Point(1, 0), 0xFF);
			test(bmd, new Point(1, 1), 0xFF, otherBmd, new Point(1, 1), 0xFF);
			trace("");

			// Testing bmd against bitmap, same as above
			trace("/// hitTest with bitmap");
			test(bmd, new Point(0, 0), 0, bitmap, new Point(0, 0), 0);
			test(bmd, new Point(1, 1), 0xFF, bitmap, new Point(3, 3), 0xA0);
			test(bmd, new Point(2, 1), 0xA0, bitmap, new Point(1, 3), 0xA0);
			test(bmd, new Point(3, 1), 0xA0, bitmap, new Point(1, 2), 0xFF);
			trace("");

			// Testing bmd against rect, offsets the rect by -firstPoint and then looks for any opaque pixel inside rect
			trace("/// hitTest with rect");
			test(bmd, new Point(0, 0), 0xA0, new Rectangle(2, 2, 2, 2));
			test(bmd, new Point(0, 0), 0xFF, new Rectangle(0, 0, 3, 4));
			test(bmd, new Point(0, 0), 0xFF, new Rectangle(2, 2, 1, 1));
			test(bmd, new Point(2, 2), 0xFF, new Rectangle(4, 4, 1, 1));
			test(bmd, new Point(-1, 0), 0xA0, new Rectangle(2, 2, 5, 5));
			test(bmd, new Point(-10, 10), 0x00, new Rectangle(0, 0, 1, 1));
			trace("");

			// Testing bmd against point, offsets the point by -firstPoint and then checks if that pixel is opaque
			trace("/// hitTest with point");
			test(bmd, new Point(0, 0), 0xA0, new Point(2, 2));
			test(bmd, new Point(0, 0), 0xFF, new Point(0, 0));
			test(bmd, new Point(0, 0), 0xFF, new Point(2, 2));
			test(bmd, new Point(2, 2), 0xFF, new Point(4, 4));
			test(bmd, new Point(3, 3), 0xFF, new Point(-1, -1));
			test(bmd, new Point(-1, -1), 0xA0, new Point(2, 2));
			test(bmd, new Point(-1, -1), 0xA0, new Point(0, 0));
			test(bmd, new Point(-10, -10), 0x00, new Point(0, 0));
			trace("");

			trace("/// Error cases")

			try {
				test(bmd, new Point(0, 0), 0x00, bmd, null);
			} catch (error: Error) {
				trace("- Error " + error.errorID);
			}

			try {
				test(bmd, new Point(0, 0), 0x00, {});
			} catch (error: Error) {
				trace("- Error " + error.errorID);
			}
		}

		// BMD looks like: ('-' is no alpha, 'x' is 0xA0, 'X' is 0xFF)
		/*   0 1 2 3 4
		 * 0 - - - - -
		 * 1 - x x x -
		 * 2 - x X x -
		 * 3 - x x x -
		 * 4 - - - - -
	 	 */
		function createImage():BitmapData {
			var bmd: BitmapData = new BitmapData(5, 5, true, 0);
			for (var x = 1; x <= 3; x++) {
				for (var y = 1; y <= 3; y++) {
					bmd.setPixel32(x, y, 0xA0FFFFFF);
				}
			}
			bmd.setPixel32(2, 2, 0xFFFFFFFF);
			return bmd;
		}

		function formatPoint(point: Point): String {
			if (point) {
				return "new Point(" + point.x + ", " + point.y + ")";
			} else {
				return "null";
			}
		}

		function formatRectangle(rect: Rectangle): String {
			if (rect) {
				return "new Rectangle(" + rect.x + ", " + rect.y + ", " + rect.width + ", " + rect.height + ")";
			} else {
				return "null";
			}
		}

		function formatObject(bmd: BitmapData, object: Object): String {
			if (object === bmd) {
				return "bmd";
			} else if (object is Point) {
				return formatPoint(object as Point);
			} else if (object is Rectangle) {
				return formatRectangle(object as Rectangle);
			} else if (object is BitmapData) {
				return "otherBitmapData";
			} else if (object is Bitmap) {
				return "otherBitmap";
			} else if (object === null) {
				return "null";
			} else {
				return "{}";
			}
		}

		function test(bmd: BitmapData, firstPoint:Point, firstAlphaThreshold:uint, secondObject:Object, secondBitmapDataPoint:Point = null, secondAlphaThreshold:uint = 1) {
			trace("// bmd.hitTest(" + formatPoint(firstPoint) + ", " + firstAlphaThreshold + ", " + formatObject(bmd, secondObject) + ", " + formatPoint(secondBitmapDataPoint) + ", " + secondAlphaThreshold + ")");
			trace(bmd.hitTest(firstPoint, firstAlphaThreshold, secondObject, secondBitmapDataPoint, secondAlphaThreshold));
			trace("");
		}
	}
	
}
