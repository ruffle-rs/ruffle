package {
	import flash.display.Stage;
	import flash.display.Sprite;

	public class Loadable {
		public function Loadable(stage: Stage) {
			trace("Hello from loaded SWF!");

			var circle:Sprite = new Sprite();
			circle.graphics.beginFill(0xFFCC00);
			circle.graphics.drawCircle(50, 50, 50);

			stage.addChild(circle);
		}
	}
}