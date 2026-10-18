package {
	import flash.display.Loader;
	import flash.display.Shape;
	import flash.display.Sprite;
	import flash.events.Event;
	import flash.events.MouseEvent;
	import flash.geom.Rectangle;
	import flash.net.URLRequest;

	[SWF(width="500", height="400")]
	public class Test extends Sprite {
		public function Test() {
			var loader:Loader = new Loader();
			addChild(loader);
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e:Event):void {
				// ScrollRect (100,100,300,200) shows the loaded content from
				// (100,100) in the loader's own (0,0)-(300,200); the content is
				// far larger, so every side of the viewport clips it.
				loader.x = 100;
				loader.y = 100;
				loader.scrollRect = new Rectangle(100, 100, 300, 200);
			});
			loader.load(new URLRequest("./loadable.swf"));

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

		private function addMarker(color:uint, x:Number, y:Number):void {
			var marker:Shape = new Shape();
			marker.graphics.beginFill(color);
			marker.graphics.drawRect(x - 5, y - 5, 10, 10);
			marker.graphics.endFill();
			addChild(marker);
		}
	}
}
