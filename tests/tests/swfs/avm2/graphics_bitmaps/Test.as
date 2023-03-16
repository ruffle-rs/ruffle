package  {

import flash.display.BitmapData;
import flash.display.MovieClip;
	import flash.display.Graphics;
	import flash.display.Shape;
import flash.geom.Matrix;


public class Test extends MovieClip {
		
		
		public function Test() {
			simple_shapes_fill();
			simple_shapes_stroke();
		}

		public function simple_shapes_fill() {
			var child:Shape = new Shape();
			var matrix: Matrix = new Matrix();
			matrix.rotate(Math.PI / 4);
			var bmd: BitmapData = new BitmapData(20, 20, true, 0xFFFFFFFF);
			bmd.noise(0);

			child.graphics.beginBitmapFill(bmd);
			child.graphics.drawRect(5, 5, 15, 20);
			child.graphics.beginBitmapFill(bmd, matrix);
			child.graphics.drawRoundRect(30, 5, 15, 20, 10);
			child.graphics.beginBitmapFill(bmd, null, false);
			child.graphics.drawCircle(50, 15, 10);
			child.graphics.beginBitmapFill(bmd, null, true, true);
			child.graphics.drawEllipse(70, 5, 10, 20);
			child.graphics.endFill();
			addChild(child);
		}

		public function simple_shapes_stroke() {
			var child:Shape = new Shape();
			var matrix: Matrix = new Matrix();
			matrix.rotate(Math.PI / 4);
			var bmd: BitmapData = new BitmapData(20, 20, true, 0xFFFFFFFF);
			bmd.noise(0);

			child.graphics.lineBitmapStyle(bmd);
			child.graphics.drawRect(5, 5, 15, 20);
			child.graphics.lineStyle(4);
			child.graphics.lineBitmapStyle(bmd, matrix);
			child.graphics.drawRoundRect(30, 5, 15, 20, 10);
			child.graphics.lineStyle(10);
			child.graphics.lineBitmapStyle(bmd, null, false);
			child.graphics.drawCircle(50, 15, 10);
			child.graphics.lineBitmapStyle(bmd, null, true, true);
			child.graphics.drawEllipse(70, 5, 10, 20);
			child.graphics.endFill();

			child.y = 100;
			addChild(child);
		}
	}
	
}
