package  {
	
	import flash.display.MovieClip;
	import flash.events.Event;
	
	
	public class MyChild extends MovieClip {
		
		
		public function MyChild() {
			trace("Constructed MyChild");
			this.addEventListener(Event.ADDED, this.onAdded);
		}
	
		private function onAdded(event: Event) {
			trace("In MyChild.onAdded - this.parent.getChildAt(1) = " + this.parent.getChildAt(1));
			trace("Child added! Running this.parent.gotoAndStop(3)");

			MovieClip(this.parent).gotoAndStop(3);
			trace("Child done");
		}
	}
	
}
