package {
    import flash.display.Sprite;
    import flash.geom.Matrix3D;
    import flash.geom.Orientation3D;
    import flash.geom.Vector3D;

    public class Test extends Sprite {
        public function Test() {
            recompose(Math.PI / 2, 0, 0);
            recompose(0, Math.PI, 0);
            recompose(0, 0, Math.PI / 2);
            recompose(Math.PI / 2, Math.PI, 0);
            recompose(0.1, 1 / 3, 2);
            recompose(1e-40, 16777217, 0);
            recompose(1e39, 0, 0);

            var h:Number = Math.SQRT1_2;
            recompose(h, 0, 0, h, Orientation3D.QUATERNION);
            recompose(0.5, 0.5, 0.5, 0.5, Orientation3D.QUATERNION);
            recompose(0.1, 0.3, 0.5, Math.sqrt(1 - 0.35), Orientation3D.QUATERNION);

            recompose(1, 0, 0, Math.PI, Orientation3D.AXIS_ANGLE);
            recompose(0, 1, 0, Math.PI / 2, Orientation3D.AXIS_ANGLE);
            recompose(0.6, 0, 0.8, 1 / 3, Orientation3D.AXIS_ANGLE);
            recompose(0.1, 0.2, 0.3, 2, Orientation3D.AXIS_ANGLE);
            recompose(-2, 0, 0, 3, Orientation3D.AXIS_ANGLE);
            recompose(0, 0, 0, 1, Orientation3D.AXIS_ANGLE);
            recompose(1e-30, 1e-30, 0, 1, Orientation3D.AXIS_ANGLE);
            recompose(1e20, 0, 0, 1, Orientation3D.AXIS_ANGLE);
            recompose(NaN, 0, 0, 1, Orientation3D.AXIS_ANGLE);
            recompose(1, 0, 0, Infinity, Orientation3D.AXIS_ANGLE);
        }

        private function recompose(x:Number, y:Number, z:Number, w:Number = 0,
                orientation:String = "eulerAngles"):void {
            var matrix:Matrix3D = new Matrix3D();
            matrix.recompose(new <Vector3D>[
                new Vector3D(1, 2, 3),
                new Vector3D(x, y, z, w),
                new Vector3D(1, 2, 0.5)
            ], orientation);
            trace(orientation + " (" + x + ", " + y + ", " + z + ", " + w + "): " + matrix.rawData);
        }
    }
}
