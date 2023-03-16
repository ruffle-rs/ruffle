package  {

	import flash.display.CapsStyle;
	import flash.display.JointStyle;
	import flash.display.LineScaleMode;
	import flash.display.MovieClip;
	import flash.display.Graphics;
	import flash.display.Shape;

	
	public class Test extends MovieClip {
		
		
		public function Test() {
			trapezoid();
			dashes();
			// cubicCircle(); // currently broken in ruffle at time of writing
			curveCircle();
		}
		
		function trapezoid() {
			var trapezoid:Shape = new Shape();

			trapezoid.graphics.lineStyle(10, 0xFFD700, 1, false, LineScaleMode.VERTICAL,
				CapsStyle.NONE, JointStyle.MITER, 10);

			trapezoid.graphics.moveTo(100, 100);

			trapezoid.graphics.lineTo(120, 50);
			trapezoid.graphics.lineTo(200, 50);
			trapezoid.graphics.lineTo(220, 100);
			trapezoid.graphics.lineTo(100, 100);

			this.addChild(trapezoid);
		}

		function dashes() {
			var shape:Shape = new Shape();
			shape.graphics.lineStyle(3, 0x990000, 0.25, false,
				LineScaleMode.NONE, CapsStyle.SQUARE);

			shape.graphics.moveTo(10, 20);
			shape.graphics.lineTo(20, 20);
			shape.graphics.moveTo(30, 20);
			shape.graphics.lineTo(50, 20);
			shape.graphics.moveTo(60, 20);
			shape.graphics.lineTo(80, 20);
			shape.graphics.moveTo(90, 20);
			shape.graphics.lineTo(110, 20);
			shape.graphics.moveTo(120, 20);
			shape.graphics.lineTo(130, 20);
			this.addChild(shape);
		}

		function cubicCircle() {
			var shape:Shape = new Shape();

			shape.graphics.beginFill(0x0000FF);
			shape.graphics.moveTo(250, 0);
			shape.graphics.cubicCurveTo(275, 0, 300, 25, 300, 50);
			shape.graphics.cubicCurveTo(300, 75, 275, 100, 250, 100);
			shape.graphics.cubicCurveTo(225, 100, 200, 75, 200, 50);
			shape.graphics.cubicCurveTo(200, 25, 225, 0, 250, 0);
			shape.graphics.endFill();

			shape.x = 100;

			this.addChild(shape);
		}

		function curveCircle() {
			var shape:Shape = new Shape();

			shape.graphics.beginFill(0x00FF00);
			shape.graphics.moveTo(250, 0);
			shape.graphics.curveTo(300, 0, 300, 50);
			shape.graphics.curveTo(300, 100, 250, 100);
			shape.graphics.curveTo(200, 100, 200, 50);
			shape.graphics.curveTo(200, 0, 250, 0);
			shape.graphics.endFill();

			shape.x = 100;
			shape.y = 100;

			this.addChild(shape);
		}
	}
	
}
