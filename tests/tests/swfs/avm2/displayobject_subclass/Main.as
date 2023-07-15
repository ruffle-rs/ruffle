package {
	import flash.display.DisplayObject;
	
	public class Main {
		public function Main() {
			try {
				new MyDisplayObjectSubclass();
			} catch (e) {
				trace("Caught err: " + e);
			}
			try {
				new DisplayObject();
			} catch (e) {
				trace("Caught err: " + e);
			}
		}
	}
}