package {
	import flash.text.StaticText;
	import flash.display.DisplayObjectContainer;
	public class Test {
		public function Test(parent: DisplayObjectContainer) {
			try {
				new StaticText();
			} catch (e:Error) {
				trace("Caught error: " + e);
				trace("e.errorID: " + e.errorID);
			}
		
			var child = parent.getChildAt(0);
			trace("Child: " + child);
			
			// FIXME - uncomment this when Ruffle implements it
			//trace("Text:");
			//trace(child.text);
		}
	}
}