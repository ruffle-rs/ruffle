package {
	import flash.geom.Matrix3D;
	import flash.geom.Vector3D;
	import flash.geom.Matrix;

	public class Test {
		public function Test() {
			var mat = new Matrix3D();
			trace("Mat:\n" + mat.rawData);
			mat.appendScale(1, 2, 3);
			trace("after appendScale(1, 2, 3):\n" + mat.rawData);;
			
			mat = new Matrix3D(Vector.<Number>([
				1, 2, 3, 4,
				5, 6, 7, 8,
				9, 10, 11, 12,
				13, 14, 15, 16
			]));
			
			trace("Mat:\n" + mat.rawData);
			
			trace("mat.position = " + mat.position);
			trace("// set mat.position = new Vector3D(12, 13, 14)");
			mat.position = new Vector3D(12, 13, 14);
			trace("mat.position = " + mat.position);
			
			trace("after mat.prependTranslation(-1, 0, 2)");
			mat.prependTranslation(-1, 0, 2);
			trace("mat.position = " + mat.position);
			
			trace("Mat:\n" + mat.rawData);
			
			trace("after mat.prepend(mat):");
			mat.prepend(mat);
			trace(mat.rawData);
			
			var other = new Matrix3D();
			other.copyFrom(mat);
			
			trace("Other:");
			trace(other.rawData);
			
			var out = new Vector.<Number>();
			out.length = 20;
			mat.copyRawDataTo(out, 1, false);
			
			trace("Out: " + out);
			
			mat.copyRawDataTo(out, 2, true);
			
			trace("Out: " + out);

			var v:Vector3D = new Vector3D(1, 2, 3, 4);
			var vOut:Vector3D;
			trace("mat.transformVector(v):");
			vOut = mat.transformVector(v);
			trace(vOut.x, vOut.y, vOut.z, vOut.w);

			var vecs:Vector.<Number> = Vector.<Number>([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
			var vecsOut:Vector.<Number> = new Vector.<Number>();
			trace("mat.transformVectors(vecs, vecsOut):");
			mat.transformVectors(vecs, vecsOut);
			trace(vecsOut);

			var vecsOutFixed:Vector.<Number> = new Vector.<Number>(vecs.length, true);
			trace("mat.transformVectors(vecs, vecsOutFixed):");
			mat.transformVectors(vecs, vecsOutFixed);
			trace(vecsOutFixed);

			var vecsOutFixedTooSmall:Vector.<Number> = new Vector.<Number>(4, true);
			trace("mat.transformVectors(vecs, vecsOutFixedTooSmall):");
			try {
				mat.transformVectors(vecs, vecsOutFixedTooSmall);
			} catch (e) {
				trace(e);
			}

			trace("mat.transformVectors(null, vecsOut):");
			try {
				mat.transformVectors(null, vecsOut);
			} catch (e) {
				trace(e);
			}

			trace("mat.transformVectors(vecs, null):");
			try {
				mat.transformVectors(vecs, null);
			} catch (e) {
				trace(e);
			}

			trace("mat.deltaTransformVector(v):");
			vOut = mat.deltaTransformVector(v);
			trace(vOut.x, vOut.y, vOut.z, vOut.w);
			
			var tooShort = new Matrix3D(Vector.<Number>([1, 2]));
			trace("Too short: " + tooShort.rawData);
			
			var tooLong = new Matrix3D(Vector.<Number>([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]));
			trace("Too long: " + tooLong.rawData);
			
			var modified = Vector.<Number>([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
			var newMat = new Matrix3D(modified);
			trace("Before modification: " + newMat.rawData);
			modified[0] = 99999;
			trace("After modification: " + newMat.rawData);
			
			var newMat = new Matrix3D(Vector.<Number>([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]));
			var col = new Vector3D();
			for each (var i in [0, 1, 2, 3]) {
				newMat.copyColumnTo(i, col);
				trace("Column: " + col + " w=" + col.w);
			}
			try {
				newMat.copyColumnTo(4, col);
			} catch (e) {
				trace("Column 4: " + e);
			}
		
			var row = new Vector3D();
			for each (var i in [0, 1, 2, 3]) {
				newMat.copyRowTo(i, row);
				trace("Row: " + row + " w=" + row.w);
			}
			try {
				newMat.copyRowTo(4, row);
			} catch (e) {
				trace("Row 4: " + e);
			}
		
			var row0 = new Vector3D(100, 200, 300, 400);
			var row1 =  new Vector3D(500, 600, 700, 800);
			var row2 =  new Vector3D(900, 1000, 1100, 1200);
			var row3 = new Vector3D(1300, 1400, 1500, 1600);
		
			newMat.copyRowFrom(0, row0);
			newMat.copyRowFrom(1, row1);
			newMat.copyRowFrom(2, row2);
			newMat.copyRowFrom(3, row3);
		
			try {
				newMat.copyRowFrom(4, row3)
			} catch (e) {
				trace("Copy from row 4: " + e);
			}
		
			trace("After row copies: " + newMat.rawData);
		
			newMat.prependRotation(90, Vector3D.X_AXIS);
			trace("After prependRotation: " + newMat.rawData);

			newMat.prependScale(1, 2, 3);
			trace("After prependScale: " + newMat.rawData);


			trace("// copyColumnFrom tests");
			var columnMatrix:Matrix3D = new Matrix3D(Vector.<Number>([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
			col = new Vector3D(3, 4, 5, 6);
			for each(i in [0, 1, 2, 3]) {
				columnMatrix.copyColumnFrom(i, col);
				trace("Matrix raw data: " + columnMatrix.rawData);
			}
			try {
				columnMatrix.copyColumnFrom(4, col);
			} catch(e) {
				trace("Column 4: " + e);
			}
		}
	}
}
