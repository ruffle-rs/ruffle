package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	
	public class Main extends MovieClip {
		
		public var runIt = true;
		public var myOrphan = new MyOrphan();
		
		public function Main() {
			var self = this;
			this.addEventListener(Event.ENTER_FRAME, function(e) {
				trace("Main - enterFrame in frame " + self.currentFrame);
				if (self.currentFrame == 2 && self.runIt) {
					self.runIt = false;
					self.dumpChildren();
					trace("Running self.myOrphan.gotoAndStop(3)");
					self.myOrphan.gotoAndStop(3);
					trace("Finished self.myOrphan.gotoAndStop(3)");
					self.dumpChildren();
				}
			});
			this.addEventListener(Event.FRAME_CONSTRUCTED, function(e) {
				trace("Main - frameConstructed");
			});
		}
	
		private function dumpChildren() {
			trace("Main Children: " + this.numChildren);
			for (var i = 0; i < this.numChildren; i++) {
				trace(i + ": " + this.getChildAt(i));
			}
		}
	}
	
}
