package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	
	public class MyOtherChild extends MovieClip {
		
		
		public function MyOtherChild() {
			this.addEventListener(Event.ENTER_FRAME, this.onEnterFrame);
			this.addEventListener(Event.FRAME_CONSTRUCTED, this.onFrameConstructed);
			trace("Calling MyOtherChild super()")
			super();
			trace("Called MyOtherChild super()");
		}
	
		private function onEnterFrame(e) {
			trace("MyOtherChild onEnterFrame");
		}
	
		private function onFrameConstructed(e) {
			trace("MyOtherChild onFrameConstructed");
		}
	}
	
}
