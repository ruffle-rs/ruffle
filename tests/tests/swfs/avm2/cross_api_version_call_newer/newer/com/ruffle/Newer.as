package com.ruffle {
	import flash.display.MovieClip;
	import flash.events.Event;

	public class Newer extends MovieClip {
		public function Newer() {
			trace("Initialized Newer with parent: " + this.parent + " Event = " + Event["WORKER_STATE"]);
			try {
				this.gotoAndPlay("badFrame");
			} catch (e) {
				// FIXME - print the entire error when Ruffle matches Flash Player
				trace("Caught error (truncated): " + e.toString().slice(0, 27));
			}
		}
	
		public function childPublicMethod(target: Object) {
			trace("Newer.childPublicMethod: Calling parentPublicMethod() on " + target);
			target.parentPublicMethod();
			trace("Newer.childPublicMethod: Called parentPublicMethod() on " + target);
		}
	
		AS3 function childAS3Method(target: Object) {
			trace("Newer.childAS3Method: Calling parentAS3Method() on " + target);
			target.parentAS3Method();
			trace("Newer.childAS3Method: Called parentAS3Method() on " + target);
		}
	}
}