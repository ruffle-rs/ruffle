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
			
			trace("circle.localToGlobal(new Point(10, 20)) = " + circle.localToGlobal(new Point(10, 20)));
			trace("circle.globalToLocal(new Point(10, 20)) = " + circle.globalToLocal(new Point(10, 20)));
			trace("Set circle.transform.matrix")
			
			circle.transform.matrix = new Matrix(1.3, 0.2, 0.1, 1.2, 40, 50);			
			
			trace("circle.scrollRect = " + circle.scrollRect);
						
			// FIXME - uncomment this when Ruffle implements 'localToGlobal'
			//trace("circle.getBounds(stage) = " + circle.getBounds(stage));
			trace("circle.localToGlobal(new Point(10, 20)) = " + circle.localToGlobal(new Point(10, 20)));
			trace("circle.globalToLocal(new Point(10, 20)) = " + circle.globalToLocal(new Point(10, 20)));
			trace("circle.hitTestPoint(400, 400, false) = " + circle.hitTestPoint(400, 400, false));
			trace("circle.hitTestPoint(400, 400, true) = " + circle.hitTestPoint(400, 400, true));

			// Test rounding behavior
			
			circle.scrollRect = new Rectangle(0.2, 0.2, 0.3, 0.3); // (x=0, y=0, w=0, h=0)
			trace(circle.scrollRect);

			circle.scrollRect = new Rectangle(1.2, 1.2, 0.3, 0.3); // (x=1, y=1, w=1, h=1)
			trace(circle.scrollRect);

			circle.scrollRect = new Rectangle(2.2, 2.2, 0.3, 0.3); // (x=2, y=2, w=0, h=0)
			trace(circle.scrollRect);			

			circle.scrollRect = new Rectangle(50, 60, 50, 100);
			
			// We haven't rendered a new frame yet, so even though scrollRect has updated,
			// the results of hitTestPoint should be the same
			trace("circle.scrollRect = " + circle.scrollRect);
			trace("circle.localToGlobal(new Point(10, 20)) = " + circle.localToGlobal(new Point(10, 20)));
			trace("circle.globalToLocal(new Point(10, 20)) = " + circle.globalToLocal(new Point(10, 20)));
			trace("circle.hitTestPoint(400, 400, false) = " + circle.hitTestPoint(400, 400, false));
			trace("circle.hitTestPoint(400, 400, true) = " + circle.hitTestPoint(400, 400, true));
			trace("circle.transform.concatenatedMatrix = " + circle.transform.concatenatedMatrix);
						
			var scrollChild = new Sprite();
			scrollChild.graphics.beginFill(0x0000ff);
			scrollChild.graphics.drawCircle(150, 150, 100);
			scrollChild.graphics.endFill();
			scrollChild.scrollRect = new Rectangle(100, 100, 60, 70);
			circle.addChild(scrollChild);
			
			trace("scrollChild.localToGlobal(new Point(10, 20)) = " + scrollChild.localToGlobal(new Point(10, 20)));
			trace("scrollChild.globalToLocal(new Point(10, 20)) = " + scrollChild.globalToLocal(new Point(10, 20)));
			trace("Change scrollChild coordinates");
			scrollChild.transform.matrix = new Matrix(1.3, 0.2, 0.2, 1.2, 60, 70);
			trace("scrollChild.localToGlobal(new Point(10, 20)) = " + scrollChild.localToGlobal(new Point(10, 20)));
			trace("scrollChild.globalToLocal(new Point(10, 20)) = " + scrollChild.globalToLocal(new Point(10, 20)));
			
			trace("scrollChild.transform.concatenatedMatrix = " + scrollChild.transform.concatenatedMatrix);
			
			var normalChild = new Sprite();
			normalChild.graphics.beginFill(0x00ff00);
			normalChild.graphics.drawCircle(100, 100, 20);
			normalChild.graphics.endFill();
			normalChild.transform.matrix = new Matrix(1.1, 0.1, 0.3, 1.1, -30, -10);
			circle.addChild(normalChild);

			// Flash only applies the scrollRect after a render, so wait 50ms to ensure
			// that we've rendered a frame.
			var timer = new Timer(50, 1);
			timer.addEventListener(TimerEvent.TIMER, function(e) {
				trace("After 50ms delay");
				trace("circle.scrollRect = " + circle.scrollRect);;
				// FIXME - uncomment these lines when Ruffle implement 'getBounds'
				//trace("circle.getBounds(stage) = " + circle.getBounds(stage));
				trace("circle.localToGlobal(new Point(10, 20)) = " + circle.localToGlobal(new Point(10, 20)));
				trace("circle.globalToLocal(new Point(10, 20)) = " + circle.globalToLocal(new Point(10, 20)));
				trace("scrollChild.localToGlobal(new Point(10, 20)) = " + scrollChild.localToGlobal(new Point(10, 20)));
				trace("scrollChild.globalToLocal(new Point(10, 20)) = " + scrollChild.globalToLocal(new Point(10, 20)));
				trace("circle.transform.concatenatedMatrix = " + circle.transform.concatenatedMatrix);
				trace("scrollChild.transform.concatenatedMatrix = " + scrollChild.transform.concatenatedMatrix);
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