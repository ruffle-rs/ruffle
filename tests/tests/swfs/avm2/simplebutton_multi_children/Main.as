package {
	import flash.events.Event;
	import flash.display.DisplayObjectContainer;
	import flash.display.MovieClip;

	public class Main extends MovieClip {
		public function Main() {
			this.addEventListener("enterFrame", this.onEnterFrame);
		}
	
		private function onEnterFrame(e: Event) {
			var button = new MyButton();
			printChildren("upState", button.upState);
			printChildren("downState", button.downState);
			printChildren("overState", button.overState);
			printChildren("hitTestState", button.hitTestState);			
		}
	
		private function printChildren(name: String, container: DisplayObjectContainer) {
			trace(name + ": " + container);
			for (var i = 0; i < container.numChildren; i++) {
				trace("Child " + i + ": " + container.getChildAt(i) + " text: " + container.getChildAt(i)["text"]);
			}
		}
	}
}