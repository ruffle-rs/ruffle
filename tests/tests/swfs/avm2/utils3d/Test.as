package {
	import flash.geom.Matrix3D;
	import flash.geom.Utils3D;
	import flash.geom.Vector3D;

	public class Test {
		public function Test() {
			var vec = new Vector3D(1.0, 2.0, 3.0, 4.0);
			var mat = new Matrix3D(Vector.<Number>([
				100, 200, 300, 400,
				500, 600, 700, 800,
				900, 1000, 1100, 1200,
				1300, 1400, 1500, 1600
			]));
			
			var projected = Utils3D.projectVector(mat, vec);
			trace("Projected: " + projected + " w = " + projected.w);
			
			var verts = Vector.<Number>([100, 200, 300, 400, 500, 600]);
			var projectedVerts = Vector.<Number>([]);
			var uvts = Vector.<Number>([]);
			
			Utils3D.projectVectors(mat, verts, projectedVerts, uvts);
			trace("After bad project:");
			trace("projectedVerts: " + projectedVerts);
			trace("UVTs: " + uvts);
			
			uvts = Vector.<Number>([1000, 2000, 3000,
			4000, 5000, 6000,
			5, 6]); // Deliberately missing a final z coord
			Utils3D.projectVectors(mat, verts, projectedVerts, uvts);
			trace("After good project:");
			trace("projectedVerts: " + projectedVerts);
			trace("UVTs: " + uvts);
		}
	}
}