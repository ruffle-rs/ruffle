package {
	import flash.display.MovieClip;
	import flash.events.Event;
	import flash.events.EventDispatcher;
	
	public class EventWatcher {
		private var name: String;
		private var source: EventDispatcher;
		
		public function EventWatcher(name: String, source: EventDispatcher) {
			super();
			this.name = name;
			this.source = source;
			this.setup();
		}
		
		function trace_event(event: Event) {
			trace(this.name + ":" + event);
			trace("///event.currentTarget");
			trace(event.currentTarget);
			trace("///event.target");
			trace(event.target);
		}
		
		public function setup() {
			this.source.addEventListener(Event.SOUND_COMPLETE, this.trace_event);
		}
		
		public function destroy() {
			this.source.removeEventListener(Event.SOUND_COMPLETE, this.trace_event);
		}
	}
}