package  {
	import flash.display.MovieClip;
	import flash.events.Event;
	
	public class LoaderEvents extends MovieClip {
		public function LoaderEvents() {
			var self = this;
			trace('constructed ' + this.name);
			addEventListener(Event.ADDED, function(e) {
				trace('added ' + e.target['name']);
			});
		}
	}
}
