
package {
	import flash.display.MovieClip;
	import flash.geom.Matrix;

	public class Test {
		public function Test(main: MovieClip) {
			
			var props = ["rotation", "x", "y", "scaleX", "scaleY"];
			// FIXME - we should also be testing Infinity and -Infinity here,
			// but those give very weird values back in the matrix,
			// and I havne't yet figured out how to reproduce them. Hopefully,
			// there are no SWFs relying on the behavior.
			for each (var prop in props) {
				var clip = new MovieClip();
				clip.transform.matrix = new Matrix(2, 0, 4, 0, 5, 6);
				trace("Setting initial matrix: " + clip.transform.matrix);
				trace("Initial value: clip[" + prop + "] = " + clip[prop]);
				tryValue(clip, prop, NaN);
				
				clip = new MovieClip();
				var newMat = new Matrix(2, 0, 4, 0, 5, 6);
				clip.transform.matrix = newMat;
				trace("Setting initial matrix: " + clip.transform.matrix);
				trace("Initial value: clip[" + prop + "] = " + clip[prop]);
				tryValue(clip, prop, NaN);
				
				clip = new MovieClip();
				var newMat = new Matrix(7, 0, 9, 0, 11, 12);
				clip.transform.matrix = newMat;
				trace("Setting initial matrix: " + clip.transform.matrix);
				trace("Initial value: clip[" + prop + "] = " + clip[prop]);
				tryValue(clip, prop, NaN);
			}
		}
	}
}

import flash.display.MovieClip;

function tryValue(clip:MovieClip, prop:String, value:*) {
	clip[prop] = value;
	trace("Setting clip[" + prop + "] = " + value + " gave: " + clip[prop]);
	trace("Matrix: " + clip.transform.matrix);
}