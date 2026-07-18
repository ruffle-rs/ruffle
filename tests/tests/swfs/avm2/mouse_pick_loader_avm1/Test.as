package  {
	
	import flash.display.MovieClip;
	import flash.display.Loader;
	import flash.events.Event;
	import flash.events.FocusEvent;
	import flash.net.URLRequest;
	import flash.display.Sprite;
	import flash.events.KeyboardEvent;
	import flash.ui.Keyboard;
	
	public class Test extends MovieClip {
		
		var avm1Loader:Loader = new Loader();
		
		var avm1Container:Sprite = new Sprite();

		public function Test() {
			super();
			
			avm1Container.name = "avm1Container";
			addChild(avm1Container);

			avm1Loader.contentLoaderInfo.addEventListener("complete", onAVM1Loaded);
			avm1Loader.load(new URLRequest("avm1.swf"));
			
			stage.addEventListener("click", function(e) {
				trace("Clicked on:", e.target, "(" + e.target.name + ")");
			});
			
			stage.addEventListener("keyDown", onKeyPress);
		}
		
		public function onKeyPress(e:KeyboardEvent) {
			if (e.keyCode == Keyboard.SHIFT) {
				avm1Container.mouseEnabled = !avm1Container.mouseEnabled;
				trace("avm1Container.mouseEnabled is now", avm1Container.mouseEnabled);
			} else if (e.keyCode == Keyboard.SPACE) {
				avm1Container.mouseChildren = !avm1Container.mouseChildren;
				trace("avm1Container.mouseChildren is now", avm1Container.mouseChildren);
			}
		}
		
		public function onAVM1Loaded(e:Event):void {
			avm1Container.addChild(avm1Loader);
		}

	}
	
}
