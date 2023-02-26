package {
	import flash.display.DisplayObject;

	public class MyDisplayObjectSubclass extends DisplayObject {
		public function MyDisplayObjectSubclass() {
			trace("ERR - constructor should not be reachable");
			super();
		}
	}
}