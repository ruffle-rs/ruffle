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
			
			this.stage.addEventListener(MouseEvent.CLICK, function (e) {
				trace("Clicked at: " + e.stageX + " " + e.stageY + " on: " + e.target + " (" + e.target.name + ")");
			});
		
			var loaderWrapper = new Sprite();
			loaderWrapper.name = "LoaderWrapper";
		
			var firstLoader = new Loader();
			firstLoader.name = "firstLoader";
			firstLoader.load(new URLRequest("image.png"));
			loaderWrapper.addChild(firstLoader);

			var secondLoader = new Loader();
			secondLoader.name = "secondLoader";
			secondLoader.load(new URLRequest("image.png"));
			secondLoader.y = 150;
			secondLoader.mouseEnabled = false;
			loaderWrapper.addChild(secondLoader);
		
			var thirdLoader = new Loader();
			thirdLoader.name = "thirdLoader";
			thirdLoader.load(new URLRequest("image.png"));
			thirdLoader.y = 300;
			thirdLoader.mouseChildren = false;
			loaderWrapper.addChild(thirdLoader);
		
			var fourthLoader = new Loader();
			fourthLoader.name = "fourthLoader";
			fourthLoader.load(new URLRequest("image.png"));
			fourthLoader.y = 450;
			fourthLoader.mouseEnabled = false;
			fourthLoader.mouseChildren = false;
			loaderWrapper.addChild(fourthLoader);
			
			this.addChild(loaderWrapper);
		}
	}
	
}
