package  {
	
	import flash.display.MovieClip;
	import flash.display.BitmapData;
	import flash.geom.Matrix;
	import flash.display.Bitmap;
	import flash.geom.Point;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var a = new BitmapData(100, 100, false, 0xFFFFFF);
			var matrix = new Matrix();
			matrix.rotate((45/180) * Math.PI);
			matrix.translate(50, -20);
			
			var b = new BitmapData(100, 100, false, 0x000000);
			b.draw(a, matrix);
			var bitmap = new Bitmap(b);
			addChild(bitmap);
			
			var c = new BitmapData(100, 100, false, 0xFF0000);
			c.copyPixels(b, b.rect, new Point(0, 0));
			bitmap = new Bitmap(c);
			bitmap.x = 120;
			addChild(bitmap);
		}
	}
	
}
