package {
	import flash.display.MovieClip;
	import flash.display.Sprite;

	public class Test {
		public function Test(main:MovieClip) {
			var target = new Sprite();
			target.graphics.beginFill(0x0);
			target.graphics.drawCircle(50, 50, 40);
			target.graphics.endFill();
			
			var mask = new Sprite();
			mask.graphics.beginFill(0xFF0000);
			mask.graphics.drawCircle(20, 20, 50);
			mask.graphics.endFill();
			
			target.mask = mask;
			main.addChild(target);
			main.addChild(mask);
			
			target.name = "Target"
			mask.name = "Mask"
			
			main.stage.addEventListener("mouseDown", function(e) {
				trace("mouseDown: target=" + e.target.name + " stageX=" + e.stageX + " stageY=" + e.stageY);
			});
		}
	}
}