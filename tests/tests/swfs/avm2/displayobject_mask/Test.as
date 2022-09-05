package {
	import flash.display.Stage;

	public class Test {
		import flash.text.TextField;
		import flash.display.Sprite;
		import flash.events.MouseEvent;
		
		public function Test(stage: Stage) {

			var circle = new Sprite();
			circle.graphics.beginFill(0xFF0000);
			circle.graphics.drawCircle(50, 50, 40);
			stage.addChild(circle);
			
			var rect = new Sprite();
			rect.graphics.beginFill(0x00FF00);
			rect.graphics.drawRect(0, 0, 60, 60);
			stage.addChild(rect);
			
			trace("Initial mask: " + circle.mask);
			circle.mask = rect;
			trace("Mask after set: " + circle.mask);
			trace("rect == circle.mask: ", rect == circle.mask);
		}
	}
}