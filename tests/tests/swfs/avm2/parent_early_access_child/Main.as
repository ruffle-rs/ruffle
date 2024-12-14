package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	
	public class Main extends MovieClip {
		
		public var firstChild: MovieClip;
		public var secondChild: MovieClip;
		
		public function Main() {
			this.addEventListener(Event.ENTER_FRAME, this.onEnterFrame);
			this.addEventListener(Event.FRAME_CONSTRUCTED, this.onFrameConstructed);
			this.addEventListener(Event.EXIT_FRAME, this.onExitFrame);
		}
	
		private function onEnterFrame(e) {
			trace("Main.onEnterFrame: this.firstChild = " + this.firstChild + " this.secondChild = " + this.secondChild);
		}
	
		private function onFrameConstructed(e) {
			trace("Main.onFrameConstructed: this.firstChild = " + this.firstChild + " this.secondChild = " + this.secondChild);
		}
	
		private function onExitFrame(e) {
			trace("Main.onExitFrame: this.firstChild = " + this.firstChild + " this.secondChild = " + this.secondChild);
		}
	}
	
}
