

package {
	import flash.geom.Matrix3D;
	import flash.geom.Vector3D;
	
	public class Test {
		
		public function Test() {
			var matrices:Array = [
				new Matrix3D(), // Identity matrix
				createTranslationMatrix(10, 20, 30), // Translation only
				createScaleMatrix(2, 2, 2), // Scaling only
				createRotationMatrix(0.3, 0.5, 0.7), // Rotation only
				createRotationScaleMatrix(0.3, 0.5, 0.7, 2, 2, 2), // Rotation and scaling
				createNonUniformScaleMatrix(2, 3, 4), // Non-uniform scaling
			];

			for each (var matrix:Matrix3D in matrices) {
				trace("Original determinant: " + matrix.determinant);
				// Invert the matrix
				var invertedMatrix:Matrix3D = matrix.clone();
				invertedMatrix.invert();
				
				trace("Inverted:");
				trace(invertedMatrix.rawData);
			}
		}

		private function createTranslationMatrix(x:Number, y:Number, z:Number):Matrix3D {
			var matrix:Matrix3D = new Matrix3D();
			matrix.appendTranslation(x, y, z);
			return matrix;
		}

		private function createScaleMatrix(x:Number, y:Number, z:Number):Matrix3D {
			var matrix:Matrix3D = new Matrix3D();
			matrix.appendScale(x, y, z);
			return matrix;
		}

		private function createRotationMatrix(rx:Number, ry:Number, rz:Number):Matrix3D {
			var matrix:Matrix3D = new Matrix3D();
			matrix.appendRotation(rx, Vector3D.X_AXIS);
			matrix.appendRotation(ry, Vector3D.Y_AXIS);
			matrix.appendRotation(rz, Vector3D.Z_AXIS);
			return matrix;
		}

		private function createRotationScaleMatrix(rx:Number, ry:Number, rz:Number, sx:Number, sy:Number, sz:Number):Matrix3D {
			var matrix:Matrix3D = new Matrix3D();
			matrix.appendRotation(rx, Vector3D.X_AXIS);
			matrix.appendRotation(ry, Vector3D.Y_AXIS);
			matrix.appendRotation(rz, Vector3D.Z_AXIS);
			matrix.appendScale(sx, sy, sz);
			return matrix;
		}

		private function createNonUniformScaleMatrix(sx:Number, sy:Number, sz:Number):Matrix3D {
			var matrix:Matrix3D = new Matrix3D();
			matrix.appendScale(sx, sy, sz);
			return matrix;
		}
	}
}