package {
	import flash.display.MovieClip;
	import flash.display.Shape;
	
	public class Test {
		public function Test(main:MovieClip) {
			main.stage.scaleMode = "showAll";
			var rect = new Shape();
			rect.graphics.beginFill(0xFF0000);
			rect.graphics.drawRect(0, 0, main.stage.stageWidth, main.stage.stageHeight);
			rect.graphics.endFill();
			
			main.addChild(rect);
			
			main.stage.addEventListener("enterFrame", function(e) {
				trace("Stage enterFrame: stageWidth=" + main.stage.stageWidth + " stageHeight = " + main.stage.stageHeight);
			});
		}
	}
}