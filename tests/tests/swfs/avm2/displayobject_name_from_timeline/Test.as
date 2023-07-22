package  {
	
	import flash.display.MovieClip;
	import flash.utils.describeType;
	import flash.display.Sprite;
	
	
	public class Test extends MovieClip {
		
		
		public function Test() {
			trace("// circle.");
			trace(circle);
			trace("");
			
			trace("// circle.name");
			trace(circle.name);
			trace("");
			
			trace("// circle.name = \"square\"");
			try {
				circle.name = "square";
			} catch (e) {
				trace(e);
			}
			trace("");
			
			trace("// circle");
			trace(circle);
			trace("");
			
			trace("// circle.name");
			trace(circle.name);
			trace("");
			
			trace("// removeChild(circle)");
			removeChild(circle);
			trace("");
			
			trace("// circle.name = \"square\"");
			try {
				circle.name = "square";
			} catch (e) {
				trace(e);
			}
			trace("");
			
			trace("// addChild(circle)");
			addChild(circle);
			trace("");
			
			trace("// circle.name = \"square\"");
			try {
				circle.name = "square";
			} catch (e) {
				trace(e);
			}
			trace("");
		}
	}
	
}
