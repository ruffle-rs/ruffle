package {
	import flash.events.Event;
	import flash.display.MovieClip;

	public class Test {
		
		public function Test(main: MovieClip) {
			main.addEventListener(Event.ENTER_FRAME, function(e) {
				var child = new MyChild();
				child.gotoAndStop(2);
				
				trace("Grandchild: " + child.grandChild);
			})
		}
	}
}