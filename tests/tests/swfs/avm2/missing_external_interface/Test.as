package {
	import flash.display.MovieClip;
	import flash.external.ExternalInterface;
	
	public class Test extends MovieClip {
		public function Test() {
			try {
				ExternalInterface.call("foo");
			} catch(e) {
				trace("Caught exception from ExternalInterface.call");
				trace(e);
				trace(e.errorID);
			}
		
			try {
				ExternalInterface.addCallback("myCallback", function() {});
			} catch(e) {
				trace("Caught exception from ExternalInterface.addCallback");
				trace(e);
				trace(e.errorID);
			}

			try {
				ExternalInterface.addCallback("myCallback", null);
			} catch(e) {
				trace("Caught exception from ExternalInterface.addCallback");
				trace(e);
				trace(e.errorID);
			}

			trace("ExternalInterface.objectID: " + ExternalInterface.objectID);
		}
	}
}