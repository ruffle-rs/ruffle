package com.ruffle2 {
	import flash.display.MovieClip;
	import flash.display.Loader;
	import flash.net.URLRequest;
	import flash.events.Event;

	public class Test extends MovieClip {
		public function Test() {
			var loader = new Loader();
			var self = this;
			loader.contentLoaderInfo.addEventListener(Event.COMPLETE, function(e) {
				trace("Calling childPublicMethod on " + loader.content);
				loader.content.childPublicMethod(self);
				trace("Called childPublicMethod");
				
				trace("Calling childAS3Method on " + loader.content);
				loader.content.childAS3Method(self);
				trace("Called childAS3Method");
			})
			loader.load(new URLRequest("newer/newer.swf"));
		}
	
		public function parentPublicMethod() {
			trace("Called Test.parentPublicMethod");
		}
	
		AS3 function parentAS3Method() {
			trace("Called Test.parentAS3Method()");
		}
	}
}