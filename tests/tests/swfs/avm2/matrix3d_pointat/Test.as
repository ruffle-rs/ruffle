package {
    import flash.display.Sprite;
    import flash.geom.Matrix3D;
    import flash.geom.Vector3D;

    public class Test extends Sprite {
        private function traceVector(label:String, value:Vector3D):void {
            trace(label + ": " + value.x + ", " + value.y + ", " + value.z);
        }

        public function Test() {
            // Away3D's camera convention: its local +Z points at the target and
            // its local -Y remains the camera's up axis.
            var camera:Matrix3D = new Matrix3D();
            camera.position = new Vector3D(10, 20, 30);
            camera.pointAt(new Vector3D(50, 80, 140), Vector3D.Z_AXIS, new Vector3D(0, -1, 0));

            traceVector("away3d forward", camera.deltaTransformVector(Vector3D.Z_AXIS));
            traceVector("away3d up", camera.deltaTransformVector(new Vector3D(0, -1, 0)));
            traceVector("away3d position", camera.position);

            // Matrix3D.pointAt changes orientation only; existing scale must be
            // retained when the matrix is recomposed.
            var scaled:Matrix3D = new Matrix3D();
            scaled.appendScale(2, 3, 4);
            scaled.position = new Vector3D(-5, 6, 7);
            scaled.pointAt(new Vector3D(15, 36, 47), Vector3D.Z_AXIS, new Vector3D(0, -1, 0));
            trace("scale lengths: "
                + scaled.deltaTransformVector(Vector3D.X_AXIS).length + ", "
                + scaled.deltaTransformVector(Vector3D.Y_AXIS).length + ", "
                + scaled.deltaTransformVector(Vector3D.Z_AXIS).length);
            traceVector("scaled position", scaled.position);

            // Default object-relative axes are +Y forward and +Z up.
            var defaults:Matrix3D = new Matrix3D();
            defaults.pointAt(new Vector3D(30, 40, 50));
            traceVector("default forward", defaults.deltaTransformVector(Vector3D.Y_AXIS));
            traceVector("default up", defaults.deltaTransformVector(Vector3D.Z_AXIS));
        }
    }
}
