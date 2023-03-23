package {
	import flash.display.BitmapData;
	import flash.display.Bitmap;
	import flash.geom.Rectangle;
	import flash.display.MovieClip;

	public class Test extends MovieClip {
		var numTests: int = 0;
		
		public function Test() {
			test();
		}

		function test() {
			testFill(new Rectangle(0, 0, 0, 0));
			testFill(new Rectangle(0, 0, 5, 5));
			testFill(new Rectangle(-5, -5, 8, 8));
			testFill(new Rectangle(15, 15, -8, -8));
			testFill(new Rectangle(-10, -10, 100, 100));
			testFill(new Rectangle(0, 0, 0, 0));
		}
		
		function testFill(rect) {
			testFillOpaque(rect);
			testFillTransparent(rect);
		}
		
		function testFillOpaque(rect) {
			var bmd = new BitmapData(10, 10, false, 0x000000BB);
			bmd.fillRect(rect, 0x00FF0000);
			addBitmap(bmd);
		}
		
		function testFillTransparent(rect) {
			var bmd = new BitmapData(10, 10, true, 0x440000BB);
			bmd.fillRect(rect, 0xAAFF0000);
			addBitmap(bmd);
		}
		
		function addBitmap(bmd: BitmapData) {
			var i = this.numTests++;
			var x = Math.floor(i % 10);
			var y = Math.floor(i / 10);
			var bitmap = new Bitmap(bmd);
			bitmap.x = 20 + (x * 20);
			bitmap.y = 20 + (y * 20);
			addChild(bitmap);
		}
	}
}