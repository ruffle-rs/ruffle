package {
	import flash.display.MovieClip;

	public class UnlinkedMovieClip extends MovieClip {
		public function UnlinkedMovieClip() {
			trace("UnlinkedMovieClip before super(): this.graphics: " + this.graphics + " this.numChildren = " + this.numChildren);
			super();
			trace("UnlinkedMovieClip after super(): this.graphics: " + this.graphics + " this.numChildren = " + this.numChildren);
		}
	}
}