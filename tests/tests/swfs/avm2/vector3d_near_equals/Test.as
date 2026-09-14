package {
    import flash.display.*;
    import flash.geom.Vector3D;

    public class Test extends Sprite {
        public function Test() {
            var v1 = new Vector3D(1, 1, 1, 1);
            var v2 = new Vector3D(2, 2, 2, 2);
            var v0 = new Vector3D(0, 0, 0, 0);

            trace("v(1).nearEquals(v(2),1) = " + v(1).nearEquals(v(2),1));
            trace("v(1).nearEquals(v(2),1,true) = " + v(1).nearEquals(v(2),1,true));
            trace("v(1).nearEquals(v(2),1.1) = " + v(1).nearEquals(v(2),1.1));
            trace("v(1).nearEquals(v(2),1.1,true) = " + v(1).nearEquals(v(2),1.1,true));

            trace("v(1).nearEquals(v(0),1) = " + v(1).nearEquals(v(0),1));
            trace("v(1).nearEquals(v(0),1,true) = " + v(1).nearEquals(v(0),1,true));
            trace("v(1).nearEquals(v(0),1.1) = " + v(1).nearEquals(v(0),1.1));
            trace("v(1).nearEquals(v(0),1.1,true) = " + v(1).nearEquals(v(0),1.1,true));

            var v1 = v(1);
            var v2 = v(2);
            var v3 = v(3);
            trace(v1.nearEquals(v2,1));
            traceVectors(v1, v2);
            trace(v1.nearEquals(v2,2));
            traceVectors(v1, v2);
            trace(v1.nearEquals(v2,1,true));
            traceVectors(v1, v2);
            trace(v1.nearEquals(v2,2,true));
            traceVectors(v1, v2);

            v1.x = 2;
            v1.y = 1;
            v1.z = 2;
            v1.w = 1;
            trace(v1.nearEquals(v2,1,true));
            traceVectors(v1, v2);
            v1.x = 1;
            v1.y = 2;
            v1.z = 2;
            v1.w = 1;
            trace(v1.nearEquals(v2,1,true));
            traceVectors(v1, v2);
            v1.x = 2;
            v1.y = 2;
            v1.z = 1;
            v1.w = 1;
            trace(v1.nearEquals(v2,1,true));
            traceVectors(v1, v2);
            v1.x = 2;
            v1.y = 2;
            v1.z = 2;
            v1.w = 1;
            trace(v1.nearEquals(v2,1,true));
            traceVectors(v1, v2);

            v1.w = 10000;
            trace(v1.nearEquals(v2,1,true));
            traceVectors(v1, v2);
            v1.w = -10000;
            trace(v1.nearEquals(v2,1,true));
            traceVectors(v1, v2);

            v1.w = -1;
            trace(v1.nearEquals(v2,1.9,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v2,2,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v2,2.1,true));
            traceVectors(v1, v2);

            v1.w = -1;
            trace(v1.nearEquals(v3,2,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v3,2.9,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v3,3,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v3,3.1,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v(1.234),2,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v(1.234),2,false));
            traceVectors(v1, v2);

            // Negative tolerance.
            v1 = v(-1);
            trace(v1.nearEquals(v(-1),0,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v(-1),-1,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v(-2),-2,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v(-2),-5,true));
            traceVectors(v1, v2);
            v1.w = -1;
            trace(v1.nearEquals(v(-2),5,true));
            traceVectors(v1, v2);
        }

        function v(n:Number):Vector3D {
            return new Vector3D(n,n,n,n);
        }

        function traceVectors(v1:Vector3D, v2:Vector3D) {
            trace("  v1 = " + v1.x + "," + v1.y + "," + v1.z + "," + v1.w);
            trace("  v2 = " + v2.x + "," + v2.y + "," + v2.z + "," + v2.w);
        }
    }
}
