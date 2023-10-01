package {
	import flash.display.MovieClip;
	import flash.events.Event;
	import flash.utils.getDefinitionByName;

	public class Main extends MovieClip {
		public function Main() {
			trace("In constructor");
			root.loaderInfo.addEventListener("open", function(e) {
				trace("ERROR: Called open event!");
			});
			root.loaderInfo.addEventListener("init", function(e) {
				trace("Called init event!");
			});
			root.loaderInfo.addEventListener("complete", function(e) {
				trace("Called complete event!");
			});
		
			this.addEventListener(Event.ENTER_FRAME, function(e) {
				trace("Called enterFrame");
				try {
					trace("SecondFrameChild: " + getDefinitionByName("SecondFrameChild"));
				} catch (e) {
					trace("Caught error in Main enterFrame: " + e);
				}
			
				try {
					trace("FourthFrameChild: " + getDefinitionByName("FourthFrameChild"));
				} catch (e) {
					trace("Caught error in Main enterFrame: " + e);
				}
			});
		
			this.addEventListener(Event.FRAME_CONSTRUCTED, function(e) {
				trace("Called frameConstructed");
			})
			trace("Finished constructor");
		}
	}
}