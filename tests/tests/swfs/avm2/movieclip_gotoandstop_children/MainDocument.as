package {
	import flash.display.MovieClip;

	public class MainDocument extends MovieClip {
		public function MainDocument() {
			trace("/// this.child.gotoAndStop(2)");
			this.child.gotoAndStop(2);

			trace("/// (Grandchildren of this.child...)");
			for (var i = 0; i < this.child.numChildren; i += 1) {
				trace("Child: " + this.child.getChildAt(i).name);
			}

			this.stop();
		}
	}
}