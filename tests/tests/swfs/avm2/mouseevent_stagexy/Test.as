package {
	import flash.display.MovieClip;
	import flash.events.MouseEvent;
	import flash.events.Event;

	public class Test extends MovieClip {
		function Test() {
			this.addEventListener("FakeEvent", this.fake_event_listener);
			
			trace("/// (dispatching MouseEvent at 5.0, 1.5...)");
			this.dispatchEvent(new MouseEvent("FakeEvent", false, false, 5.0, 1.5));
			
			this.x = 32;
			this.y = 16;
			
			trace("/// (dispatching MouseEvent at 5.0, 1.5...)");
			this.dispatchEvent(new MouseEvent("FakeEvent", false, false, 5.0, 1.5));
		}
		
		public function fake_event_listener(evt: Event) {
			if (evt is MouseEvent) {
				assert_event(evt as MouseEvent);
			}
		}
	}
}

import flash.events.MouseEvent;
import flash.geom.Matrix;

function assert_event(evt: MouseEvent) {
	trace("/// evt.localX = " + evt.localX + " evt.localY = " + evt.localY + " evt.stageX = " + evt.stageX + " evt.stageY = " + evt.stageY);
	
	evt.localX = 1;
	evt.localY = 2;
	
	trace("/// set localX=1 localY=2")
	
	trace("/// evt.localX = " + evt.localX + " evt.localY = " + evt.localY + " evt.stageX = " + evt.stageX + " evt.stageY = " + evt.stageY);
	
	trace("/// modified target.x and target.y")
	
	evt.target.x = 100;
	evt.target.y = 100;
	
	trace("/// evt.localX = " + evt.localX + " evt.localY = " + evt.localY + " evt.stageX = " + evt.stageX + " evt.stageY = " + evt.stageY);
	
	trace("/// modified target.transform.matrix")
	
	evt.target.transform.matrix = new Matrix(1, 2, 3, 4, 5, 6);
	
	trace("/// evt.localX = " + evt.localX + " evt.localY = " + evt.localY + " evt.stageX = " + evt.stageX + " evt.stageY = " + evt.stageY);
}