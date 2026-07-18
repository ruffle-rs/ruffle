package {
	import flash.display.Sprite;

	public class Test extends Sprite {
		public function Test() {
			// Test splice with sneaky valueOf
			var arr:Array = ["a", "b", "c", "d"];
			var sneakyIndex:Object = {
				valueOf: function():int {
					trace("valueOf for index called");
					trace("array length during valueOf: " + arr.length);
					return 1;
				}
			};
			var sneakyCount:Object = {
				valueOf: function():int {
					trace("valueOf for count called");
					trace("array length during valueOf: " + arr.length);
					return 2;
				}
			};
			trace("before splice: " + arr);
			arr.splice(sneakyIndex, sneakyCount, "new");
			trace("after splice: " + arr);
		}
	}
}
