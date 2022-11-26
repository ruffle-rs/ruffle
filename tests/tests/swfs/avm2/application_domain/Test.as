package {
	import flash.system.ApplicationDomain;
	
	public class Test {
		public function Test() {
			try {
				ApplicationDomain.currentDomain.getDefinition("some.package.MissingClass")
			} catch (e:Error) {
				trace("Caught error: " + e);
			}
		
			try {
				ApplicationDomain.currentDomain.getDefinition("OtherMissingClass")
			} catch (e:Error) {
				trace("Caught error: " + e);
			}
		
			trace("Has definition: " +  ApplicationDomain.currentDomain.hasDefinition("flash.display.MovieClip"));
			var cls = ApplicationDomain.currentDomain.getDefinition("flash.display.MovieClip");
			trace("Got class: " + cls);
		}
	}
}