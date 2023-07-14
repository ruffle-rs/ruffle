package  {
	
	import flash.display.MovieClip;
	import flash.display.DisplayObject;
	import flash.events.Event;
	
	
	public class Main extends MovieClip {
		
		private var runIt = true;
		
		public var myOtherChild:DisplayObject;
		
		
		public function Main() {
			this.addEventListener(Event.ENTER_FRAME, this.onEnterFrame);
		}
	
		private function onEnterFrame(event: Event) {
			if (this.runIt) {
				this.runIt = false;
				trace("Enter frame!");
				this.gotoAndStop(3);
				trace("Enter frame done");
			}
		}
	}
	
}
