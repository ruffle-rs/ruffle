package {
	import flash.display.MovieClip;

	public class MainDocument extends MovieClip {
		public function MainDocument() {
			trace("/// this.gotoAndStop(1)");
			this.gotoAndStop(1);
			
			trace("/// this.gotoAndStop(2)");
			this.gotoAndStop(2);
			
			trace("/// this.gotoAndStop(1)");
			this.gotoAndStop(1);

			trace("/// this.stop()");
			this.stop();
		}
	}
}