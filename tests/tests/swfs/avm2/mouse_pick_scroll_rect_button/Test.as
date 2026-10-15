package {
	import flash.display.Shape;
	import flash.display.SimpleButton;
	import flash.display.Sprite;
	import flash.events.MouseEvent;
	import flash.geom.Rectangle;

	[SWF(width="500", height="400")]
	public class Test extends Sprite {
		public function Test() {
			var button:SimpleButton = new SimpleButton();
			button.upState = makeState(0x00FF00);
			button.overState = makeState(0xFFFF00);
			button.downState = makeState(0x0000FF);
			button.hitTestState = makeState(0xFF00FF);
			addChild(button);
			// ScrollRect (100,100,300,200) shows the button states from
			// (100,100) in the button's own (0,0)-(300,200); the hit area is
			// far larger, so every side of the viewport clips it.
			button.x = 100;
			button.y = 100;
			button.scrollRect = new Rectangle(100, 100, 300, 200);

			// Markers are plain shapes, so they never take mouse events; they
			// only make the probe points visible when replaying in Flash Player.
			// Green marks the inside region, red the outside regions.
			addMarker(0x00FF00, 250, 200); // inside
			addMarker(0xFF0000, 50, 200);  // left
			addMarker(0xFF0000, 450, 200); // right
			addMarker(0xFF0000, 250, 50);  // top
			addMarker(0xFF0000, 250, 350); // bottom

			stage.addEventListener(MouseEvent.MOUSE_DOWN, function(e:MouseEvent):void {
				trace("mouseDown: " + e.target + " at " + e.stageX + "," + e.stageY);
			});
		}

		private function makeState(color:uint):Shape {
			var state:Shape = new Shape();
			state.graphics.beginFill(color);
			state.graphics.drawRect(0, 0, 3000, 3000);
			state.graphics.endFill();
			return state;
		}

		private function addMarker(color:uint, x:Number, y:Number):void {
			var marker:Shape = new Shape();
			marker.graphics.beginFill(color);
			marker.graphics.drawRect(x - 5, y - 5, 10, 10);
			marker.graphics.endFill();
			addChild(marker);
		}
	}
}
