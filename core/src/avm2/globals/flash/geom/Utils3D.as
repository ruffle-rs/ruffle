package flash.geom {
    public class Utils3D {
        public static function projectVector(m:Matrix3D, v:Vector3D):Vector3D {
            var projected = m.transformVector(v);
            projected.x /= projected.w;
            projected.y /= projected.w;
            projected.z /= projected.w;

            return projected;
        }

        // Based on https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/geom/Utils3D.hx
        public static function projectVectors(m:Matrix3D, verts:Vector.<Number>, projectedVerts:Vector.<Number>, uvts:Vector.<Number>):void {
            var n = m.rawData;
            var x, y, z, w;
            var x1, y1, z1, w1;

            var i = 0;
            var j = 0;

            if (uvts.length < verts.length) {
                uvts.length = verts.length;
            }

            if (projectedVerts.length < (verts.length / 3) * 2) {
                projectedVerts.length = (verts.length / 3) * 2;
            }

            while (i + 2 < verts.length) {
                x = verts[i];
                y = verts[i + 1];
                z = verts[i + 2];
                w = 1;

                x1 = x * n[0] + y * n[4] + z * n[8] + w * n[12];
                y1 = x * n[1] + y * n[5] + z * n[9] + w * n[13];
                w1 = x * n[3] + y * n[7] + z * n[11] + w * n[15];

                projectedVerts[j] = x1 / w1;
                projectedVerts[j + 1] = y1 / w1;

                uvts[i + 2] = 1 / w1;

                i += 3;
                j += 2;
            }
        }
    }
}