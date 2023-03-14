package  {
	
	import flash.display.MovieClip;
	import flash.display.BitmapData
	import flash.geom.Point;
	
	
	public class Test extends MovieClip {
		
		function dumpBitmap(bm) {
			for(var x = 0; x < bm.width; x++) {
				for(var y = 0; y < bm.height; y++) {
					trace("// bitmap.getPixel32(" + x + ", " + y + ")");
					var c = bm.getPixel32(x, y);
					
					var b = c & 0xFF;
					var g = (c >> 8) & 0xFF;
					var r = (c >> 16) & 0xFF;
					var a = (c >>248) & 0xFF;
					
					trace(a + ", " + r + ", " + g + ", " + b);
				}
			}
		}
		
		
		public function Test() {
			// How is invalid operation handled
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), undefined, 0xFF000000, 0xFFFF0000, 0xFF000000, false)");
			try{
				trace(A.threshold(B, B.rect, new Point(0, 0), undefined, 0xFF000000, 0xFFFF0000, 0xFF000000, false));
			} catch (error: Error) {
				trace("! error " + error.errorID);
			}
			trace("// A.threshold(B, B.rect, new Point(0, 0), 0, 0xFF000000, 0xFFFF0000, 0xFF000000, false)");
			try{
				trace(A.threshold(B, B.rect, new Point(0, 0), 0, 0xFF000000, 0xFFFF0000, 0xFF000000, false));
			} catch (error: Error) {
				trace("! error " + error.errorID);
			}
			trace("// A.threshold(B, B.rect, new Point(0, 0), equals, 0xFF000000, 0xFFFF0000, 0xFF000000, false)");
			try{
				trace(A.threshold(B, B.rect, new Point(0, 0), "equals", 0xFF000000, 0xFFFF0000, 0xFF000000, false));
			} catch (error: Error) {
				trace("! error " + error.errorID);
			}
			
			
			// Test copysource=y
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), !=, 0xFF000000, 0xFFFF0000, 0xFF000000, true)");
			trace(A.threshold(B, B.rect, new Point(0, 0), "!=", 0xFF000000, 0xFFFF0000, 0xFF000000, true));
			dumpBitmap(A);
			
			// Normal ops
			
			// eq
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), ==, 0xFF000000, 0xFFFF0000, 0xFF000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), "==", 0xFF000000, 0xFFFF0000, 0xFF000000, false));
			dumpBitmap(A);
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), ==, 0xFFF00000, 0xFFFF0000, 0xFF000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), "==", 0xFFF00000, 0xFFFF0000, 0xFF000000, false));
			dumpBitmap(A);
			
			// neq
			var A = new BitmapData(2, 2, true, 0xFF000000);
			var B = new BitmapData(2, 2, true, 0xF0000000);
			trace("// A.threshold(B, B.rect, new Point(0, 0), !=, 0xFFF00000, 0xFFFF0000, 0xFF000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), "!=", 0xFFF00000, 0xFFFF0000, 0xFF000000, false));
			dumpBitmap(A);
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFF000000);
			trace("// A.threshold(B, B.rect, new Point(0, 0), !=, 0xFF000000, 0xFFFF0000, 0xFF000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), "!=", 0xFF000000, 0xFFFF0000, 0xFF000000, false));
			dumpBitmap(A);
			
			// lt
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), <, 0xFFF00000, 0xFFFF0000, 0xFF000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), "<", 0xFFF00000, 0xFFFF0000, 0xFF000000, false));
			dumpBitmap(A);
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), <, 0xFF000000, 0xFFFF0000, 0xF0000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), "<", 0xFF000000, 0xFFFF0000, 0xF0000000, false));
			dumpBitmap(A);
			
			// lte
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), <=, 0xFFF00000, 0xFFFF0000, 0xFF000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), "<=", 0xFFF00000, 0xFFFF0000, 0xFF000000, false));
			dumpBitmap(A);
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), <=, 0xFF000000, 0xFFFF0000, 0xF0000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), "<=", 0xFF000000, 0xFFFF0000, 0xF0000000, false));
			dumpBitmap(A);
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), <=, 0xFF000000, 0xFFFF0000, 0xFF000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), "<=", 0xFF000000, 0xFFFF0000, 0xFF000000, false));
			dumpBitmap(A);
			
			// gt
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), >, 0xFFF00000, 0xFFFF0000, 0xFF000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), ">", 0xFFF00000, 0xFFFF0000, 0xFF000000, false));
			dumpBitmap(A);
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), >, 0x10000000, 0xFFFF0000, 0xF0000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), ">", 0x10000000, 0xFFFF0000, 0xF0000000, false));
			dumpBitmap(A);
			
			// gte
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), >=, 0xFFF00000, 0xFFFF0000, 0xFF000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), ">=", 0xFFF00000, 0xFFFF0000, 0xFF000000, false));
			dumpBitmap(A);
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), >=, 0x10000000, 0xFFFF0000, 0xF0000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), ">=", 0x10000000, 0xFFFF0000, 0xF0000000, false));
			dumpBitmap(A);
			var A = new BitmapData(2, 2, true, 0x00000000);
			var B = new BitmapData(2, 2, true, 0xFFFFFFFF);
			trace("// A.threshold(B, B.rect, new Point(0, 0), >=, 0xFF000000, 0xFFFF0000, 0xFF000000, false)");
			trace(A.threshold(B, B.rect, new Point(0, 0), ">=", 0xFF000000, 0xFFFF0000, 0xFF000000, false));
			dumpBitmap(A);
			
			
			// Partially out of range pixels
			var A = new BitmapData(3, 3, true, 0xFF000000);
			var B = new BitmapData(3, 3, true, 0xFF000000);
			trace("// A.threshold(B, B.rect, new Point(1, 0), ==, 0xFF000000, 0xFFFF0000, 0xFFFFFFFF, false)");
			trace(A.threshold(B, B.rect, new Point(1, 0), "==", 0xFF000000, 0xFFFF0000, 0xFFFFFFFF, false));
			dumpBitmap(A);
			
			
			// Check for how aliasing is handled
			// var A = new BitmapData(3, 3, true, 0xFF000000);
			// trace("// A.threshold(A, A.rect, new Point(1, 0), ==, 0xFF000000, 0xFFFF0000, 0xFFFFFFFF, false)");
			// trace(A.threshold(A, A.rect, new Point(1, 0), "==", 0xFF000000, 0xFFFF0000, 0xFFFFFFFF, false));
			// dumpBitmap(A);
		}
	}
	
}
