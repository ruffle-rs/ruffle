package {
	import flash.geom.Matrix3D;
	import flash.geom.Vector3D;
	import flash.geom.Orientation3D;
	
	public class Test {
		public function Test() {
			testRecompose();
			testRecomposeWithZeroScale();
			// FIXME - we don't reproduce the NaNs that Flash gives us here.
			//testRecomposeWeird();
			testDecompose();
			testDecomposeOtherRotate();
			
			try {
				new Matrix3D().decompose("badOrientation");
			} catch (e) {
				trace("Caught error: " + e);
			}
		
			try {
				new Matrix3D().recompose(Vector.<Vector3D>([]), "badOrientation");
			} catch (e) {
				trace("Caught error: " + e);
			}
		}
	
		public function testRecompose():void {
			var translation:Vector3D = new Vector3D(1, 2, 3);
			var rotation:Vector3D = new Vector3D(4, 5, 6);
			var scale:Vector3D = new Vector3D(7, 8, 9);
			var matrix:Matrix3D = new Matrix3D();
			var vectors = Vector.<Vector3D>([translation, rotation, scale]);

			// FIXME - add back 'QUATERNION' when Ruffle properly throws an exception.
		    for each (var style in [Orientation3D.EULER_ANGLES, Orientation3D.AXIS_ANGLE, /*Orientation3D.QUATERNION*/]) {
				try {
					trace("Style: " + style);
					trace("Recompose res: " + matrix.recompose(vectors, style));
					trace("Recomposed:\n" + matrix.rawData);
				} catch (e) {
					trace("Caught error with style: " + style + " : " + e);
				}
			}
		}	
	
		public function testRecomposeWithZeroScale():void {
			var translation:Vector3D = new Vector3D(1, 2, 3);
			var rotation:Vector3D = new Vector3D(0, 0, 0);
			var scale:Vector3D = new Vector3D(0, 0, 0); // Zero scale
			var matrix:Matrix3D = new Matrix3D();
			var vectors = Vector.<Vector3D>([translation, rotation, scale]);
			matrix.recompose(vectors);
			
			trace("Recomposed zero scale:\n" + matrix.rawData);
		}
	
		function testRecomposeWeird() {
			var matrix = new Matrix3D(Vector.<Number>([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]));
			for each (var style in [Orientation3D.EULER_ANGLES, Orientation3D.AXIS_ANGLE]) {
				trace("Style: " + style);
				var decomposed = matrix.decompose(style);
				trace("Weird Decompose: " + decomposed);
				var recomposed = new Matrix3D();
				recomposed.recompose(decomposed, style);
				trace("Weird Original:");
				trace(matrix.rawData);
				trace("Weird Recomposed:");
				trace(recomposed.rawData);
			}
		}
	
		function testDecompose() {
			var matrix = new Matrix3D();
			matrix.appendRotation(1, Vector3D.Y_AXIS);
			matrix.appendTranslation(100, 200 ,300);
			matrix.appendScale(0.5, 3, 2);
			for each (var style in [Orientation3D.EULER_ANGLES, Orientation3D.AXIS_ANGLE, Orientation3D.QUATERNION]) {
				trace("Style: " + style);
				var decomposed = matrix.decompose(style);
				trace("Decompose: " + decomposed);
				var recomposed = new Matrix3D();
				recomposed.recompose(decomposed, style);
				trace("Original:");
				trace(matrix.rawData);
				trace("Recomposed:");
				trace(recomposed.rawData);
			}
		}

		function testDecomposeOtherRotate() {
			var matrix = new Matrix3D();
			matrix.appendRotation(1, new Vector3D(1, 2, 3));
			matrix.appendTranslation(100, 200 ,300);
			matrix.appendScale(0.5, 3, 2);
			for each (var style in [Orientation3D.EULER_ANGLES, Orientation3D.AXIS_ANGLE]) {
				trace("Style: " + style);
				var decomposed = matrix.decompose(style);
				trace("Decompose: " + decomposed);
			}
		}

	}
}