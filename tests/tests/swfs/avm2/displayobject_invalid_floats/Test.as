package {
	import flash.display.MovieClip;
	import flash.geom.Matrix;

	public class Test extends MovieClip {
		var props = ["rotation", "x", "y", "scaleX", "scaleY"];

		public function Test() {
			// FIXME - we should also be testing Infinity and -Infinity here,
			// but those give very weird values back in the matrix,
			// and I havne't yet figured out how to reproduce them. Hopefully,
			// there are no SWFs relying on the behavior.
			for each (var prop in props) {
				var clip = new MovieClip();
				clip.transform.matrix = new Matrix(2, 0, 4, 0, 5, 6);
				printChange(clip, prop, NaN);

				clip = new MovieClip();
				var newMat = new Matrix(2, 0, 4, 0, 5, 6);
				clip.transform.matrix = newMat;
				printChange(clip, prop, NaN);

				clip = new MovieClip();
				var newMat = new Matrix(7, 0, 9, 0, 11, 12);
				clip.transform.matrix = newMat;
				printChange(clip, prop, NaN);
			}
		}

		private function printChange(clip:MovieClip, prop:String, value:*): void {
			trace("  transform.matrix = " + clip.transform.matrix);
			var result:String = "";
			for each (var p1:String in props) {
				result += p1 + "=" + clip[p1] + ", ";
			}
			trace("  " + result);

			trace("clip[" + prop + "] = " + value);
			clip[prop] = value;

			result = "";
			for each (var p2:String in props) {
				result += p2 + "=" + clip[p2] + ", ";
			}
			trace("  " + result);
			trace("  transform.matrix = " + clip.transform.matrix);
		}
	}
}
