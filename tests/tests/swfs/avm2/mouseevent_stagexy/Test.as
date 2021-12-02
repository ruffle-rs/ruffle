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

function assert_event(evt: MouseEvent) {
	trace("/// evt.stageX");
	trace(evt.stageX);

	trace("/// evt.stageY");
	trace(evt.stageY);
}