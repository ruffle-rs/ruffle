package {
	import flash.display.Sprite;

	public class Test {
		import flash.display.DisplayObjectContainer;
		
		public function Test(main: DisplayObjectContainer) {
			trace("main = " + main + ", main.root = " + main.root);
			trace("main.loaderInfo = " + main.loaderInfo);
			trace("main.root.loaderInfo = " + main.root.loaderInfo);
			
			var child = new Sprite();
			trace("Not on stage: child.loaderInfo = " + child.loaderInfo);
			main.addChild(child);
			trace("On stage: child.loaderInfo = " + child.loaderInfo);
			trace("child.loaderInfo === child.root.loaderInfo:", child.loaderInfo === child.root.loaderInfo);
			trace("child.loaderInfo.content === child.root.loaderInfo.content:", child.loaderInfo.content === child.root.loaderInfo.content);
			
			var grandChild = new Sprite();
			child.addChild(grandChild);
			
			trace("On stage: grandChild.loaderInfo = " + grandChild.loaderInfo);
			trace("grandChild.loaderInfo === grandChild.root.loaderInfo:", grandChild.loaderInfo === grandChild.root.loaderInfo);
			trace("grandChild.loaderInfo === main.loaderInfo:", grandChild.loaderInfo === main.loaderInfo);
		}
	}
}