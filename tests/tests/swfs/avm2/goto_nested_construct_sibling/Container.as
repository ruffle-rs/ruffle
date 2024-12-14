package {
	import flash.display.MovieClip;
	import flash.events.Event;
	
	public class Container extends MovieClip {
		public var firstChild;
		public var secondChild;
		public var thirdChildDummy;
		
		public static var INSTANCE: Container = null;
		
		public function Container() {
			INSTANCE = this;
			this.addEventListener(Event.FRAME_CONSTRUCTED, this.frameConstructed);
			this.addFrameScript(0, function() {
				trace("Running Container framescript");
			})
			
			trace("Calling Container super()");
			super();
			trace("Called Container super()");
		}

		private function frameConstructed(e) {
			trace("Container frameConstructed: " + e);
		}
	}
}