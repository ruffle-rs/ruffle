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
		}
	}
}