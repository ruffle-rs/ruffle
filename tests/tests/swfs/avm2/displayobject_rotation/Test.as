package {

import flash.display.Sprite;

public class Test extends Sprite {
	public function Test() {
		trace("//this.rotation;");
		trace(this.rotation);

		trace("//this.rotation = 0.0;");
		this.rotation = 0.0;

		trace("//this.rotation;");
		trace(this.rotation);

		trace("//this.rotation = 360;");
		this.rotation = 360;

		trace("//this.rotation;");
		trace(this.rotation);

		trace("//this.rotation = -360;");
		this.rotation = -360;

		trace("//this.rotation;");
		trace(this.rotation);

		trace("//this.rotation = 240;");
		this.rotation = 240;

		trace("//this.rotation;");
		trace(this.rotation);

		trace("//this.rotation = -240;");
		this.rotation = -240;

		trace("//this.rotation;");
		trace(this.rotation);

		trace("//this.rotation = -500;");
		this.rotation = -500;

		trace("//this.rotation;");
		trace(this.rotation);

		trace("//this.rotation = 180;");
		this.rotation = 180;

		trace("//this.rotation;");
		trace(this.rotation);

		trace("//this.rotation = -180;");
		this.rotation = -180;

		trace("//this.rotation;");
		trace(this.rotation);

		trace("//(accumulating ramp from -180 to -175 in 1/60 steps);");

		for (var i = 0; i < (5 * 60); i += 1) {
			this.rotation += 1/60;
			trace(this.rotation);
		}

		trace("//(accumulating ramp from -175 to -165 in 1/30 steps);");

		for (i = 0; i < (10 * 30); i += 1) {
			this.rotation += 1/30;
			trace(this.rotation);
		}

		trace("//(accumulating ramp from -165 to -155 in 1/15 steps);");

		for (i = 0; i < (10 * 15); i += 1) {
			this.rotation += 1/15;
			trace(this.rotation);
		}

		trace("//(accumulating ramp from -155 to -145 in 1/10 steps);");

		for (i = 0; i < (10 * 10); i += 1) {
			this.rotation += 1/10;
			trace(this.rotation);
		}

		trace("//(accumulating ramp from -145 to -135 in 1/5 steps);");

		for (i = 0; i < (10 * 5); i += 1) {
			this.rotation += 1/5;
			trace(this.rotation);
		}

		trace("//(accumulating ramp from -135 to -125 in 1/3 steps);");

		for (i = 0; i < (10 * 3); i += 1) {
			this.rotation += 1/3;
			trace(this.rotation);
		}

		trace("//(accumulating ramp from -125 to -110 in 1/2 steps);");

		for (i = 0; i < (15 * 2); i += 1) {
			this.rotation += 1/2;
			trace(this.rotation);
		}

		trace("//(accumulating ramp from -110 to 180 in 1/1 steps);");

		for (i = 0; i < (290 * 1); i += 1) {
			this.rotation += 1/1;
			trace(this.rotation);
		}
	}
}

}
