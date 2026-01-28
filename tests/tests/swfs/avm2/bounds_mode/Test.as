package {

	import flash.display.DisplayObject;
	import flash.display.MovieClip;
	import flash.geom.Rectangle;

	public class Test extends MovieClip {

		public function Test() {
			trace("===== Frame 1 (initial) =====");
			testBounds();
			testWidthHeight(this.hitbox);

			trace("\n===== Frame 2 =====");
			this.hitbox.gotoAndStop(2);
			testBounds();
			testWidthHeight(this.hitbox);

			trace("\n===== Frame 3 =====");
			this.hitbox.gotoAndStop(3);
			testBounds();
			testWidthHeight(this.hitbox);

			trace("\n===== Frame 4 =====");
			this.hitbox.gotoAndStop(4);
			testBounds();
			testWidthHeight(this.hitbox);

			trace("\n===== Frame 5 =====");
			this.hitbox.gotoAndStop(5);
			testBounds();
			testWidthHeight(this.hitbox);

			trace("\n===== TextField bounds =====");
			testTextFieldBounds();
			testWidthHeight(this.textbox);
			
		}

		function testBounds():void {
			// getBounds - includes strokes
			trace("// hitbox.getBounds(this)");
			trace(this.hitbox.getBounds(this));
			
			trace("// hitbox.getBounds(hitbox)");
			trace(this.hitbox.getBounds(this.hitbox));
			
			// getRect - excludes strokes
			trace("// hitbox.getRect(this)");
			trace(this.hitbox.getRect(this));
			
			trace("// hitbox.getRect(hitbox)");
			trace(this.hitbox.getRect(this.hitbox));
			
			// pixelBounds - stage pixel coordinates
			trace("// hitbox.transform.pixelBounds");
			trace(this.hitbox.transform.pixelBounds);
			
			// width/height getters
			trace("// hitbox.width");
			trace(this.hitbox.width);
			trace("// hitbox.height");
			trace(this.hitbox.height);
		}
		
		function testTextFieldBounds():void {
			// getBounds
			trace("// textbox.getBounds(this)");
			trace(this.textbox.getBounds(this));
			
			trace("// textbox.getBounds(textbox)");
			trace(this.textbox.getBounds(this.textbox));
			
			// getRect
			trace("// textbox.getRect(this)");
			trace(this.textbox.getRect(this));
			
			trace("// textbox.getRect(textbox)");
			trace(this.textbox.getRect(this.textbox));
			
			// pixelBounds
			trace("// textbox.transform.pixelBounds");
			trace(this.textbox.transform.pixelBounds);
			
			// width/height
			trace("// textbox.width");
			trace(this.textbox.width);
			trace("// textbox.height");
			trace(this.textbox.height);
		}
		
		function testWidthHeight(displayObject: DisplayObject):void {
			trace("\n===== Width/Height setters =====");
			
			var originalWidth:Number = displayObject.width;
			var originalHeight:Number = displayObject.height;
			trace("// Original hitbox.width");
			trace(originalWidth);
			trace("// Original hitbox.height");
			trace(originalHeight);
			trace("// Original hitbox.scaleX");
			trace(displayObject.scaleX);
			trace("// Original hitbox.scaleY");
			trace(displayObject.scaleY);
			
			// Set width
			trace("// hitbox.width = 200");
			displayObject.width = 200;
			trace("// hitbox.width");
			trace(displayObject.width);
			trace("// hitbox.scaleX");
			trace(displayObject.scaleX);
			trace("// hitbox.getBounds(this)");
			trace(displayObject.getBounds(this));
			
			// Set height
			trace("// hitbox.height = 100");
			displayObject.height = 100;
			trace("// hitbox.height");
			trace(displayObject.height);
			trace("// hitbox.scaleY");
			trace(displayObject.scaleY);
			trace("// hitbox.getBounds(this)");
			trace(displayObject.getBounds(this));
			
			// Set to 0
			trace("// hitbox.width = 0");
			displayObject.width = 0;
			trace("// hitbox.width");
			trace(displayObject.width);
			trace("// hitbox.scaleX");
			trace(displayObject.scaleX);
			
			// Restore
			displayObject.scaleX = 1;
			displayObject.scaleY = 1;
		}
	}
}
