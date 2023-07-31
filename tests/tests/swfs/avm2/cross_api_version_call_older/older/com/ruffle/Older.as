package com.ruffle {
	import flash.display.MovieClip;
	import flash.events.Event;

	public class Older extends MovieClip {
		public function Older() {
			trace("Initialized Older with parent: " + this.parent + " Event = " + Event["WORKER_STATE"]);
			try {
				this.gotoAndPlay("badFrame");
			} catch (e) {
				// FIXME - print the entire error when Ruffle matches Flash Player
				trace("Caught error (truncated): " + e.toString().slice(0, 27));
			}
		}
	
		public function childPublicMethod(target: Object) {
			trace("Older.childPublicMethod: Calling parentPublicMethod() on " + target);
			target.parentPublicMethod();
			trace("Older.olderPublicMethod: Called parentPublicMethod() on " + target);
		}
	
		AS3 function childAS3Method(target: Object) {
			trace("Older.childAS3Method: Calling parentAS3Method() on " + target);
			target.parentAS3Method();
			trace("Older.childAS3Method: Called parentAS3Method() on " + target);
		}
	}
}