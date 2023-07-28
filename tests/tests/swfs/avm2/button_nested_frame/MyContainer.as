package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	
	public class MyContainer extends MovieClip {
		
		public var myButton;
		public var otherChild;
		public var myOtherButton;
		public var myOtherChild;
		public var dumbButton;
		
		public function MyContainer() {
			var self = this;
			addFrameScript(0, function() {
				trace("Running MyContainer framescript: this.myButton = " + self.myButton + " this.otherChild = " + self.otherChild + " this.myOtherButton = " + self.myOtherButton + " this.myOtherChild = " + self.myOtherChild + " this.dumbButton = " + self.dumbButton);
			})
			this.addEventListener(Event.FRAME_CONSTRUCTED, function(e) {
				var button = self.getChildAt(0);
				trace("MyContainer frameConstructed: this.myButton = " + this.myButton + " button = " + button + " button.visible = " + button.visible);
			});
			trace("Calling MyContainer super: this.getChildAt(0) = " + this.getChildAt(0));
			super();
			trace("Called MyContainer super: this.getChildAt(0) = " + this.getChildAt(0) + " this.myButton = " + this.myButton);
		}
	}
	
}
