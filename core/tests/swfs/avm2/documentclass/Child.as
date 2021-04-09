package {
	import flash.display.MovieClip;

	public class Child extends MovieClip {
		public function Child() {
			trace("/// Child constructor");
			trace("/// this.b");
			trace(this.b);
		}
	}
}