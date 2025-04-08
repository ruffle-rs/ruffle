package {
	import flash.display.MovieClip;
	import flash.display.Shape;
	import flash.display.Sprite;
	import flash.events.MouseEvent;

	public class Test extends MovieClip {
		public function Test() {
			var child = new Shape();
			child.graphics.beginFill(0xFF0000);
			child.graphics.drawRect(0, 0, 100, 100);
			child.graphics.endFill();

			var childHolder = new Sprite();
			childHolder.name = "childHolder";
			childHolder.addChild(child);

			this.addChild(childHolder);
			
			this.stage.addEventListener(MouseEvent.CLICK, function (e) {
				trace("Clicked at: " + e.stageX + " " + e.stageY + " on: " + e.target + " (" + e.target.name + ")");

				e.target.stage.mouseChildren = false;
			});
		}
	}
}
