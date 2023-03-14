
package {
	import flash.display.MovieClip;

	public class Test {
		public function Test(main:MovieClip) {
			trace("Constructing ParentClip");
			var parentClip = new ParentClip();
			trace("Constructed ParentClip");
			
			trace("Adding to main");
			main.addChild(parentClip);
			trace("Added to main");
		}
	}
}