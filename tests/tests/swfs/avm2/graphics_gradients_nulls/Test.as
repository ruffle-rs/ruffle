// compiled with mxmlc

package  {
	import flash.display.MovieClip;
	import flash.display.Graphics;
	import flash.geom.Matrix;

	public class Test extends MovieClip {

		public function Test() {
	        var matrix = new Matrix();
	        matrix.createGradientBox(200, 200, 0, 100, 100);
	        graphics.beginGradientFill(
	            "linear",
	            [0, 0xFFFFFF, 0, 0xFFFFFF, 0],
	            null, //[1, 1, 1, 1, 1],
	            null, //[0, 64, 128, 192, 255],
	            matrix
	        );
	        graphics.drawRect(100, 100, 200, 200);
	        graphics.endFill();
		}
	}
}
