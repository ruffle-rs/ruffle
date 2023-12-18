package  {
	
	import flash.display.MovieClip;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
		}
	}
	
}

import flash.utils.getDefinitionByName;

const HIDDEN_CLASSES: Array = ["flash.net.DatagramSocket", "flash.desktop.IFilePromise"];

for each (var klass in HIDDEN_CLASSES) {
	try {
		var obj = getDefinitionByName(klass);
		trace("ERROR: " + klass + " is accessible from Flash Player");
	} catch (e) {
		trace(klass + " is inaccessible from Flash Player");
	}
}