package {
	import flash.display.Shape;
	import flash.display.Sprite;
	import flash.events.MouseEvent;
	import flash.geom.Rectangle;
	import flash.text.TextField;
	import flash.text.TextFieldType;

	[SWF(width="500", height="400")]
	public class Test extends Sprite {
		public function Test() {
			var container:Sprite = new Sprite();
			var tf:TextField = new TextField();
			tf.width = 3000;
			tf.height = 3000;
			tf.selectable = true;
			tf.wordWrap = false;
			tf.type = TextFieldType.INPUT;
			tf.background = true;
			tf.backgroundColor = 0;
			tf.textColor = 0xFF0000;
			tf.text = "scrollRect clip test";

			container.addChild(tf);
			addChild(container);
			// Placing the container at (100,100) with scrollRect (100,100,300,200)
			// scrolls the field so its content from (100,100) shows in the
			// container's own (0,0)-(300,200). The field is far larger than the
			// window, so every side of the viewport clips live content.
			container.x = 100;
			container.y = 100;
			container.scrollRect = new Rectangle(100, 100, 300, 200);

			// Markers are plain shapes, so they never take mouse events; they
			// only make the probe points visible when replaying in Flash Player.
			// Green marks the inside region, red the eight outside regions.
			addMarker(0x00FF00, 250, 200); // inside
			addMarker(0xFF0000, 50, 200);  // left
			addMarker(0xFF0000, 450, 200); // right
			addMarker(0xFF0000, 250, 50);  // top
			addMarker(0xFF0000, 250, 350); // bottom
			addMarker(0xFF0000, 50, 50);   // top-left
			addMarker(0xFF0000, 450, 50);  // top-right
			addMarker(0xFF0000, 50, 350);  // bottom-left
			addMarker(0xFF0000, 450, 350); // bottom-right

			stage.addEventListener(MouseEvent.MOUSE_MOVE, logEvent);
			stage.addEventListener(MouseEvent.MOUSE_DOWN, logEvent);
			stage.addEventListener(MouseEvent.MOUSE_WHEEL, logEvent);
		}

		private function addMarker(color:uint, x:Number, y:Number):void {
			var marker:Shape = new Shape();
			marker.graphics.beginFill(color);
			marker.graphics.drawRect(x - 5, y - 5, 10, 10);
			marker.graphics.endFill();
			addChild(marker);
		}

		private function logEvent(e:MouseEvent):void {
			trace(e.type + ": " + e.target + " at " + e.stageX + "," + e.stageY);
		}
	}
}
