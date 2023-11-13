package  {
	import flash.display.MovieClip;
	import flash.events.KeyboardEvent;

public class Test extends MovieClip {
		public function Test() {
			text.addEventListener(KeyboardEvent.KEY_DOWN, onKeyDown);
			text.addEventListener(KeyboardEvent.KEY_UP, onKeyUp);

			// Magic phrase is expected by "playAndMonitor", tells the test that it's ready
			trace("Hello from Flash!");
		}

		function onKeyDown(event: KeyboardEvent) {
			trace("onKeyDown");
			trace("event.charCode = " + event.charCode);
			trace("event.keyCode = " + event.keyCode);
			trace("");
		}

		function onKeyUp(event: KeyboardEvent) {
			trace("onKeyUp");
			trace("event.charCode = " + event.charCode);
			trace("event.keyCode = " + event.keyCode);
			trace("");
		}
	}
}
