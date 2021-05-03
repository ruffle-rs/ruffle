package {
	import flash.display.MovieClip;

	public class Main extends MovieClip {
		public function Main() {
			trace("/// Main constructor");
			trace("/// this.a");
			trace(this.a);
			trace("/// this.a.b");
			trace(this.a.b);
		}
	}
}