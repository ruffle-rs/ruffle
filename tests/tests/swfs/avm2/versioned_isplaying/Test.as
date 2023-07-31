package {
	import flash.display.MovieClip;

	public class Test extends MovieClip {
		public function Test() {
			this.isPlaying;
			this["isPlaying"];
		}
	
		public function get isPlaying():Boolean {
			trace("Custom non-override isPlaying");
			return false;
		}
	}
}