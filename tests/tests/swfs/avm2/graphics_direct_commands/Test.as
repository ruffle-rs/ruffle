package {

	import flash.display.CapsStyle;
	import flash.display.JointStyle;
	import flash.display.LineScaleMode;
	import flash.display.MovieClip;
	import flash.display.Graphics;
	import flash.display.Shape;
	import flash.display.GraphicsPathCommand;

	public class Test extends MovieClip {

		public function Test() {
			trapezoid();
			dashes();
			cubicCircle();
			curveCircle();
			drawPath();
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

			shape.graphics.beginFill(0xFF0000FF);

			var partial = new Shape();
			// These two commands should get overwritten by copyFrom
			partial.graphics.drawCircle(0, 0, 30);
			partial.graphics.beginFill(0xFFFF0000);
			partial.graphics.copyFrom(shape.graphics);
			partial.graphics.drawCircle(0, 0, 10);
			partial.y = 100;

			shape.graphics.lineTo(130, 20);
			this.addChild(shape);
			this.addChild(partial);
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

		function drawPath() {
			var shape:Shape = new Shape();

			var commands:Vector.<int> = new Vector.<int>(10, true);

			commands[0] = GraphicsPathCommand.MOVE_TO;
			commands[1] = GraphicsPathCommand.WIDE_LINE_TO;
			commands[2] = GraphicsPathCommand.LINE_TO;
			commands[3] = GraphicsPathCommand.LINE_TO;
			commands[4] = GraphicsPathCommand.LINE_TO;
			commands[5] = GraphicsPathCommand.CURVE_TO;
			commands[6] = GraphicsPathCommand.WIDE_MOVE_TO;
			commands[7] = GraphicsPathCommand.LINE_TO;
			commands[8] = GraphicsPathCommand.LINE_TO;
			commands[9] = GraphicsPathCommand.LINE_TO;
			

			var coords:Vector.<Number> = new Vector.<Number>(26, true);
			// MOVE_TO
			coords[0] = 66; // x
			coords[1] = 10; // y

			// WIDE_LINE_TO
			coords[2] = 0;
			coords[3] = 0;
			coords[4] = 23;
			coords[5] = 127;

			// LINE_TO
			coords[6] = 122;
			coords[7] = 50;

			// LINE_TO
			coords[8] = 10;
			coords[9] = 49;

			// LINE_TO
			coords[10] = 109;
			coords[11] = 127;

			// CURVE_TO
			coords[12] = 0;
			coords[13] = 0;
			coords[14] = 100;
			coords[15] = 100;
			
			// WIDE_MOVE_TO
			coords[16] = 0;
			coords[17] = 0;
			coords[18] = 50;
			coords[19] = 50;
			
			// LINE_TO
			coords[20] = 80;
			coords[21] = 80;
			
			// LINE_TO
			coords[22] = 120;
			coords[23] = 120;
			
			// LINE_TO
			coords[24] = 0;
			coords[25] = 90;

			shape.graphics.beginFill(0x003366);
			shape.graphics.drawPath(commands, coords);

			shape.y = 200;
			this.addChild(shape);
		}
	}

}
