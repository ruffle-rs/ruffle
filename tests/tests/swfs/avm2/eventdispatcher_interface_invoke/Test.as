package  {
	
	import flash.display.MovieClip;
	import flash.events.IEventDispatcher;
	import flash.events.Event;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			this.invokeDispatcher(new MovieClip());
		}
	
		public function invokeDispatcher(dispatch: IEventDispatcher) {
			// This method is invoked on the interface, not a concrete class
			dispatch.dispatchEvent(new Event("myEvent"));
			trace("Called dispatchEvent!");
		}
	}
	
}
