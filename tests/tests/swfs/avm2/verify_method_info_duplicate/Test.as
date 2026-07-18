package {
	import flash.display.*;

	public class Test extends Sprite {
		public function Test() {
			try {
				hello();
			} catch (e) {
				trace(e.getStackTrace());
			}
		}

		public function hello():void {
			trace("hello called");
		}
	}
}
