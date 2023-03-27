package  {
	
	import flash.display.MovieClip;
	import flash.display.BitmapData;
	import flash.display.Bitmap;
	import flash.geom.Matrix;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var mc = new MovieClip();
			var bmd = new BitmapData(100, 100, true, 0xA0A0A0A0);
			var bitmap = new Bitmap(bmd);
			var matrix = new Matrix();
			matrix.translate(50, 50);
			mc.addChild(bitmap);
			bmd.draw(mc, matrix);
			
			this.addChild(mc);
		}
	}
	
}
