package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	
	public class Main extends MovieClip {
		
		public var buttonHolder;
		
		public function Main() {
			var self = this;
			this.addFrameScript(0, this.frame1, 1, this.frame2);
			this.addEventListener(Event.FRAME_CONSTRUCTED, function(e) {
				trace("Main frameConstructed: this.buttonHolder = " + self.buttonHolder);
			});
		
			this.addEventListener(Event.EXIT_FRAME, function(e) {
				trace("Main exitFrame: this.buttonHolder = " + self.buttonHolder);
			});
		
			this.addEventListener(Event.ADDED, function(e) {
				trace("Main added: " + e + " e.target = " + e.target + " e.target.name = " + e.target.name);
			});
		
			this.addEventListener(Event.ADDED_TO_STAGE, function(e) {
				trace("Main addedToStage: " + e + " e.target = " + e.target + " e.target.name = " + e.target.name);
			});
		
			super();
		}
	
		private function frame1() {
			trace("Main framescript 1");
		}
	
		private function frame2() {
			trace("Main framescript 2: this.buttonHolder = " + this.buttonHolder);
		}
	}
	
}
