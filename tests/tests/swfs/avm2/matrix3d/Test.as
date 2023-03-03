package {
	import flash.geom.Matrix3D;
	public class Test {
		public function Test() {
			var mat = new Matrix3D();
			trace("Mat:\n" + mat.rawData);
			mat.appendScale(1, 2, 3);
			trace("after appendScale(1, 2, 3):\n" + mat.rawData)
		}
	}
}