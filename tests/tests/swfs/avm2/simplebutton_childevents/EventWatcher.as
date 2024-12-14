package {
	import flash.display.MovieClip;
	import flash.events.Event;
	
	public class EventWatcher extends MovieClip {
		public function EventWatcher() {
			this.setup();
			super();
		}
		
		function trace_event(event: Event) {
			trace(this.name + ":" + event + " target: " + event["target"]);
			if (event.target) {
				var parent_parent = "<missing>";
				if (event.target.parent) {
					parent_parent = event.target.parent.parent;
				}
				trace("target.stage: " + event.target.stage + " target.name: " + event.target.name + " target.parent: " + event.target.parent + " target.parent.parent: " + parent_parent);
			}
		}
		
		public function setup() {
			this.addEventListener(Event.ENTER_FRAME, this.trace_event);
			this.addEventListener(Event.EXIT_FRAME, this.trace_event);
			this.addEventListener(Event.ADDED, this.trace_event);
			this.addEventListener(Event.ADDED_TO_STAGE, this.trace_event);
			this.addEventListener(Event.FRAME_CONSTRUCTED, this.trace_event);
			this.addEventListener(Event.REMOVED, this.trace_event);
			this.addEventListener(Event.REMOVED_FROM_STAGE, this.trace_event);
			this.addEventListener(Event.RENDER, this.trace_event);
		}
		
		public function destroy() {
			this.removeEventListener(Event.ENTER_FRAME, this.trace_event);
			this.removeEventListener(Event.EXIT_FRAME, this.trace_event);
			this.removeEventListener(Event.ADDED, this.trace_event);
			this.removeEventListener(Event.ADDED_TO_STAGE, this.trace_event);
			this.removeEventListener(Event.FRAME_CONSTRUCTED, this.trace_event);
			this.removeEventListener(Event.REMOVED, this.trace_event);
			this.removeEventListener(Event.REMOVED_FROM_STAGE, this.trace_event);
			this.removeEventListener(Event.RENDER, this.trace_event);
		}
	}
}