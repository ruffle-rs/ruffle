package {
	import flash.display.Stage;
	import flash.display.Sprite;
	import flash.geom.Rectangle;
	import flash.geom.Point;
	import flash.events.TimerEvent;
	import flash.events.MouseEvent;
	import flash.geom.Matrix;
	import flash.utils.Timer;


	public class Test {
		public function Test(stage: Stage) {
			var circle = new Sprite();
			circle.graphics.beginFill(0xaa00aa);
			circle.graphics.drawCircle(120, 120, 100);
			circle.graphics.endFill();

			stage.addChild(circle);

			circle.transform.matrix = new Matrix(1.1, 0, 0, 1.1, 0, 0);			
			
			trace("circle.scrollRect = " + circle.scrollRect);
			// FIXME - uncomment this when Ruffle implements 'localToGlobal'
			//trace("circle.getBounds(stage) = " + circle.getBounds(stage));
			trace("circle.hitTestPoint(400, 400, false) = " + circle.hitTestPoint(400, 400, false));
			trace("circle.hitTestPoint(400, 400, true) = " + circle.hitTestPoint(400, 400, true));
			circle.scrollRect = new Rectangle(30, 40, 500, 600);
			
			// We haven't rendered a new frame yet, so even though scrollRect has updated,
			// the results of hitTestPoint should be the same
			trace("circle.scrollRect = " + circle.scrollRect);
			trace("circle.hitTestPoint(400, 400, false) = " + circle.hitTestPoint(400, 400, false));
			trace("circle.hitTestPoint(400, 400, true) = " + circle.hitTestPoint(400, 400, true));
						
			var scrollChild = new Sprite();
			scrollChild.graphics.beginFill(0x0000ff);
			scrollChild.graphics.drawCircle(150, 150, 40);
			scrollChild.graphics.endFill();
			scrollChild.scrollRect = new Rectangle(0, 0, 180, 250);
			circle.addChild(scrollChild);
			
			var normalChild = new Sprite();
			normalChild.graphics.beginFill(0x00ff00);
			normalChild.graphics.drawCircle(100, 100, 50);
			normalChild.graphics.endFill();
			stage.addChild(normalChild);

			// Flash only applies the scrollRect after a render, so wait 50ms to ensure
			// that we've rendered a frame.
			var timer = new Timer(50, 1);
			timer.addEventListener(TimerEvent.TIMER, function(e) {
				trace("After 50ms delay");
				trace("circle.scrollRect = " + circle.scrollRect);;
				// FIXME - uncomment these lines when Ruffle implement 'getBounds' and 'localToGlobal'
				//trace("circle.getBounds(stage) = " + circle.getBounds(stage));
				//trace("circle.localToGlobal(new Point(0, 0)) = " + circle.localToGlobal(new Point(0, 0)));
				trace("circle.hitTestPoint(400, 400, false) = " + circle.hitTestPoint(400, 400, false));
				trace("circle.hitTestPoint(400, 400, true) = " + circle.hitTestPoint(400, 400, true));
			})
			timer.start();

			stage.addEventListener(MouseEvent.MOUSE_MOVE, function(e:MouseEvent) {
				// Remove this 'return' to enable mouse-based debugging.
				return;
				if (circle.hitTestPoint(e.stageX, e.stageY)) {
					circle.alpha = 0.2;
				} else {
					circle.alpha = 1.0;
				}
			});	
		}
	}
}