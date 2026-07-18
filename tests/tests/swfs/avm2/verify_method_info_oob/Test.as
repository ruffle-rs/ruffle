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

		private function hello():void {
			trace("hello called");
		}
	}
}
