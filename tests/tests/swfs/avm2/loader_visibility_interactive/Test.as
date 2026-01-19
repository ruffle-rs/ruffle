package  {
	
	import flash.display.MovieClip;
	import flash.display.Shape;
	import flash.geom.Rectangle;
	import flash.net.URLRequest;
	import flash.events.MouseEvent;
	import flash.display.Loader;
	import flash.display.Sprite;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			var child = new Shape();
			child.graphics.beginFill(0xFF0000);
			child.graphics.drawRect(0, 0, 100, 100);
			child.graphics.endFill();
			this.addChild(child);
			
		
			var loaderWrapper = new Sprite();
			loaderWrapper.name = "LoaderWrapper";
		
			var firstLoader = new Loader();
			firstLoader.name = "firstLoader";
			firstLoader.load(new URLRequest("image.png"));
			firstLoader.mouseEnabled = true;
			loaderWrapper.addChild(firstLoader);
			
			firstLoader.addEventListener(MouseEvent.CLICK, function (e) {
				trace("Clicked at: " + e.stageX + " " + e.stageY + " on: " + e.target + " (" + e.target.name + ")");
				firstLoader.visible = false;
			});
			
			this.addChild(loaderWrapper);
		}
	}
	
}
