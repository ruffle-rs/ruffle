package {
	import flash.display.MovieClip;
	import flash.geom.Matrix;

	public class Test extends MovieClip {
		private var props:Array = ["rotation", "x", "y", "scaleX", "scaleY"];

		public function Test():void {
			TestAssignMatrix_AssignNaN();

			TestAssignMatrix();

			TestZeroSkew();
		}

		public function TestZeroSkew(): void {
			// The matrices are created by scaling and rotating.
			// They shouldn't have any private internal value like skew.

			var scaleVals1:Array = [0, NaN];
			var scaleVals2:Array = [1, NaN];
			var scaleVals3:Array = [1, 0];
			var rotations:Array = [0, 0.5, -0.5, 1, -1, 2, -2, 0.25, -0.25];
			for each (var prop:String in props) {
				trace("// TestZeroSkew: " + prop + " = 0");
				for(var idx1:int = 0; idx1 < 2; idx1++) {
					for(var idx2:int = 0; idx2 < 2; idx2++) {
						for(var idxr:int = 0; idxr < rotations.length; idxr++) {
							var m:Matrix = new Matrix();

							var clip:MovieClip = new MovieClip();
							m.identity();
							m.scale(scaleVals1[idx1], scaleVals1[idx2]);
							m.rotate(rotations[idxr] * Math.PI);
							clip.transform.matrix = m;
							trace(m);
							printChange(clip, prop, 0);
							trace("");

							m.identity();
							m.scale(scaleVals2[idx1], scaleVals2[idx2]);
							m.rotate(rotations[idxr] * Math.PI);
							clip.transform.matrix = m;
							trace(m);
							printChange(clip, prop, 0);
							trace("");

							m.identity();
							m.scale(scaleVals3[idx1], scaleVals3[idx2]);
							m.rotate(rotations[idxr] * Math.PI);
							clip.transform.matrix = m;
							trace(m);
							printChange(clip, prop, 0);
							trace("");
						}
					}
				}
			}
		}

		public function TestAssignMatrix_AssignNaN(): void {
			// Assign NaN to each property to observe the special behavior.

			var matVals1:Array = [2, 3];
			var matVals2:Array = [0, NaN];
			var matVals3:Array = [1, NaN];
			var matVals4:Array = [1, 0];
			var matVals5:Array = [-1, 0];

			for each (var prop:String in props) {
				trace("// TestAssignMatrix: " + prop + " = NaN");
				var clip:MovieClip = new MovieClip();

				clip.transform.matrix = new Matrix(2, 0, 4, 0, 5, 6);
				printChange(clip, prop, NaN);
				trace("");
				clip.transform.matrix = new Matrix(7, 0, 9, 0, 11, 12);
				printChange(clip, prop, NaN);
				trace("");

				for(var idx1:int = 0; idx1 < 2; idx1++) {
					for(var idx2:int = 0; idx2 < 2; idx2++) {
						for(var idx3:int = 0; idx3 < 2; idx3++) {
							for(var idx4:int = 0; idx4 < 2; idx4++) {
								clip.transform.matrix = new Matrix(matVals1[idx1], matVals1[idx2], matVals1[idx3], matVals1[idx4], 0, 0);
								printChange(clip, prop, NaN);
								trace("");
								clip.transform.matrix = new Matrix(matVals2[idx1], matVals2[idx2], matVals2[idx3], matVals2[idx4], 0, 0);
								printChange(clip, prop, NaN);
								trace("");
								clip.transform.matrix = new Matrix(matVals3[idx1], matVals3[idx2], matVals3[idx3], matVals3[idx4], 0, 0);
								printChange(clip, prop, NaN);
								trace("");
								clip.transform.matrix = new Matrix(matVals4[idx1], matVals4[idx2], matVals4[idx3], matVals4[idx4], 0, 0);
								printChange(clip, prop, NaN);
								trace("");
								clip.transform.matrix = new Matrix(matVals5[idx1], matVals5[idx2], matVals5[idx3], matVals5[idx4], 0, 0);
								printChange(clip, prop, NaN);
								trace("");
							}
						}
					}
				}
			}
		}

		public function TestAssignMatrix(): void {
			// Assign arbitrary matrix to clip.transform.matrix and verify the change in property values.
			// The matrices can be "invalid." For example, NaN, -0, Infinity, or singular matrix.

			var matVals1:Array = [0, NaN];
			var matVals2:Array = [1, NaN];
			var matVals3:Array = [1, 0];
			var matVals4:Array = [-1, 0];
			var matVals5:Array = [-0, 1];
			var matVals6:Array = [-0, 0];
			var matVals7:Array = [2, 3]; // Random numbers. Some prime numbers for no reason.
			for each (var prop1:String in props) {
				trace("// TestAssignMatrix: " + prop1 + " = finite number");
				for(var idx1:int = 0; idx1 < 2; idx1++) {
					for(var idx2:int = 0; idx2 < 2; idx2++) {
						for(var idx3:int = 0; idx3 < 2; idx3++) {
							for(var idx4:int = 0; idx4 < 2; idx4++) {
								var clip:MovieClip = new MovieClip();
								clip.transform.matrix = new Matrix(matVals1[idx1], matVals1[idx2], matVals1[idx3], matVals1[idx4], 0, 0);
								printChange(clip, prop1, 0);
								trace("");
								clip.transform.matrix = new Matrix(matVals2[idx1], matVals2[idx2], matVals2[idx3], matVals2[idx4], 0, 0);
								printChange(clip, prop1, 0);
								trace("");
								clip.transform.matrix = new Matrix(matVals3[idx1], matVals3[idx2], matVals3[idx3], matVals3[idx4], 0, 0);
								printChange(clip, prop1, 0);
								trace("");
								clip.transform.matrix = new Matrix(matVals4[idx1], matVals4[idx2], matVals4[idx3], matVals4[idx4], 0, 0);
								printChange(clip, prop1, 0);
								trace("");
								clip.transform.matrix = new Matrix(matVals5[idx1], matVals5[idx2], matVals5[idx3], matVals5[idx4], 0, 0);
								printChange(clip, prop1, 0);
								trace("");
								clip.transform.matrix = new Matrix(matVals6[idx1], matVals6[idx2], matVals6[idx3], matVals6[idx4], 0, 0);
								printChange(clip, prop1, 0);
								trace("");
								clip.transform.matrix = new Matrix(matVals6[idx1], matVals6[idx2], matVals6[idx3], matVals6[idx4], 0, 0);
								printChange(clip, prop1, -0); // -0
								trace("");
								clip.transform.matrix = new Matrix(matVals7[idx1], matVals7[idx2], matVals7[idx3], matVals7[idx4], 0, 0);
								printChange(clip, prop1, 0);
								trace("");
								clip.transform.matrix = new Matrix(matVals7[idx1], matVals7[idx2], matVals7[idx3], matVals7[idx4], 0, 0);
								printChange(clip, prop1, 5); // non zero
								trace("");
							}
						}
					}
				}
			}

			// FIXME - we should also be testing Infinity and -Infinity here,
			// but those give very weird values back in the matrix,
			// and I havne't yet figured out how to reproduce them. Hopefully,
			// there are no SWFs relying on the behavior.
			// Remove this `return` when that's fixed.
			return;

			var matValsInfinite:Array = [0, 1, NaN, Infinity, -Infinity];
			for each (var prop2:String in props) {
                trace("// TestAssignMatrix: " + prop2 + " = 0");
				for each (var l1 in matValsInfinite) {
					for each (var l2 in matValsInfinite) {
						for each (var l3 in matValsInfinite) {
							for each (var l4 in matValsInfinite) {
								var clip:MovieClip = new MovieClip();
								clip.transform.matrix = new Matrix(l1, l2, l3, l4, 0, 0);
								printChange(clip, prop2, 0);
								trace("");
							}
						}
					}
				}
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
