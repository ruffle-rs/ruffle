package 
{

	import flash.display.MovieClip;


	public class Test extends MovieClip
	{
		public function Test()
		{
			trace("// frame1");
			testHit();

			trace("// hitbox.gotoAndStop(2);");
			this.hitbox.gotoAndStop(2);
			testHit();

			trace("// hitbox.gotoAndStop(3);");
			this.hitbox.gotoAndStop(3);
			testHit();

			trace("// hitbox.gotoAndStop(4);");
			this.hitbox.gotoAndStop(4);
			testHit();

			trace("// hitbox.gotoAndStop(5);");
			this.hitbox.gotoAndStop(5);
			testHit();
		}


		function testHit()
		{
			var bounds = this.hitbox.getBounds(this);
			trace("Bounds:", bounds);
			var rect = this.hitbox.getRect(this);
			trace("Rect:", rect);
			trace("hitTest:", this.bullet.hitTestObject(this.hitbox));
			trace("hitTestPoint:" + this.bullet.hitTestPoint(80, 30, true));
			trace("hitTestPoint Non-Shape:" + this.bullet.hitTestPoint(80, 30, false));
		}

	}

}