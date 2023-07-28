package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	
	public class Main extends MovieClip {
		
		public var child;
		
		public function Main() {
			trace("Calling Main super()");
			super();
			trace("Called Main super()");
			this.addEventListener(Event.ENTER_FRAME, this.onEnterFrame);
			this.addEventListener(Event.EXIT_FRAME, this.onExitFrame);
			this.addEventListener(Event.FRAME_CONSTRUCTED, this.onFrameConstructed);
		}
	
		private function onEnterFrame(e) {
			trace("Main enter frame: " + e);
		}
	
		private function onExitFrame(e) {
			trace("Main exit frame: " + e);
		}
	
		private function onFrameConstructed(e) {
			trace("Main frame constructed: " + e);
		}
	}
	
}
