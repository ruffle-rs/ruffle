package  {

import flash.display.GradientType;
import flash.display.MovieClip;
import flash.display.Graphics;
import flash.display.Shape;
import flash.display.SpreadMethod;
import flash.geom.Matrix;


public class Test extends MovieClip {


	public function Test() {
		simple_shapes_fill();
		simple_shapes_stroke();
	}

	public function simple_shapes_fill() {
		var child:Shape = new Shape();

		var matr:Matrix = new Matrix();
		matr.createGradientBox(20, 20, 0, 0, 0);

		child.graphics.beginGradientFill(GradientType.LINEAR, [0xFF0000, 0x0000FF], [0.5, 1], [0x00, 0xFF], matr, SpreadMethod.PAD);
		child.graphics.drawRect(5, 5, 15, 20);
		child.graphics.beginGradientFill(GradientType.LINEAR, [0xFF0000, 0x00FF00], [1, 1], [0x10, 0xFF], matr, SpreadMethod.REFLECT, "linear");
		child.graphics.drawRoundRect(30, 5, 15, 20, 10);
		child.graphics.beginGradientFill(GradientType.RADIAL, [0xFF0000, 0x0000FF], [1, 1], [0x00, 0xF0], matr, SpreadMethod.PAD, "rgb", 0.75);
		child.graphics.drawCircle(50, 15, 10);
		child.graphics.beginGradientFill(GradientType.RADIAL, [0xFF0000, 0x0000FF], [0.5, 1], [0x00, 0xFF], matr, SpreadMethod.REPEAT);
		child.graphics.drawEllipse(70, 5, 10, 20);
		child.graphics.endFill();
		addChild(child);
	}

	public function simple_shapes_stroke() {
		var child:Shape = new Shape();

		var matr:Matrix = new Matrix();
		matr.createGradientBox(20, 20, 0, 0, 0);

		child.graphics.lineGradientStyle(GradientType.LINEAR, [0xFF0000, 0x0000FF], [0.5, 1], [0x00, 0xFF], matr, SpreadMethod.PAD);
		child.graphics.drawRect(5, 5, 15, 20);
		child.graphics.lineStyle(4);
		child.graphics.lineGradientStyle(GradientType.LINEAR, [0xFF0000, 0x00FF00], [1, 1], [0x10, 0xFF], matr, SpreadMethod.REFLECT, "linear");
		child.graphics.drawRoundRect(30, 5, 15, 20, 10);
		child.graphics.lineStyle(10);
		child.graphics.lineGradientStyle(GradientType.RADIAL, [0xFF0000, 0x0000FF], [1, 1], [0x00, 0xF0], matr, SpreadMethod.PAD, "rgb", 0.75);
		child.graphics.drawCircle(50, 15, 10);
		child.graphics.lineGradientStyle(GradientType.RADIAL, [0xFF0000, 0x0000FF], [0.5, 1], [0x00, 0xFF], matr, SpreadMethod.REPEAT);
		child.graphics.drawEllipse(70, 5, 10, 20);
		child.graphics.endFill();

		child.y = 100;
		addChild(child);
	}
}

}
