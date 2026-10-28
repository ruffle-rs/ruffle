package {
    import flash.display.Sprite;
    import flash.geom.Matrix3D;
    import flash.geom.Orientation3D;
    import flash.geom.Vector3D;

    public class Test extends Sprite {
        public function Test() {
            var mid:Number = 84.5 * Math.pow(2, -23);

            testQuaternion("identity", 0, 0, 0, 1);
            testQuaternion("negative identity", 0, 0, 0, -1);
            testQuaternion("half turns", 0.5, 0.5, 0.5, 0.5);

            testQuaternion("inside x", Math.sqrt(1.0073e-5), 0, 0, 1);
            testQuaternion("outside x", Math.sqrt(1.0074e-5), 0, 0, 1);
            testQuaternion("inside w", 0, 0, 0, Math.sqrt(1 - 1.0073e-5));
            testQuaternion("outside w", 0, 0, 0, Math.sqrt(1 - 1.0074e-5));

            testQuaternion("split pair", Math.sqrt(mid / 2), Math.sqrt(mid / 2), 0, 1);
            testQuaternion("split above", 0, 0, 0, Math.sqrt(1 + mid + 1e-12));
            testQuaternion("split below", 0, 0, 0, Math.sqrt(1 - (mid - 20e-12)));

            var axis:Number = Math.SQRT1_2;
            var unitW:Number = Math.sqrt(1 - axis * axis - 0.0001 * 0.0001);
            testQuaternion("tilt low", axis, 0.0001, 0, unitW * 0.99999);
            testQuaternion("tilt high", axis, 0.0001, 0, unitW * 1.00001);
            testQuaternion("tilt unit", axis, 0.0001, 0, unitW);

            var quarter:Number = Math.sqrt(0.25 + 2e-6);
            testQuaternion("four", quarter, quarter, quarter, quarter);

            testQuaternion("zero", 0, 0, 0, 0);
            testQuaternion("nan", NaN, 0, 0, 1);
            testQuaternion("infinity", Infinity, 0, 0, 0);

            testScale("zero scale", new Vector3D(0, 0, 0, 1), new Vector3D(0, 0, 0));
            testScale("negative scale", new Vector3D(0, 0, 0, 1), new Vector3D(-2, 3, -4));
            testScale("nan scale", new Vector3D(0, 0, 0, 1), new Vector3D(NaN, 1, 1));
            testScale("bad negative scale", new Vector3D(2, 0, 0, 0), new Vector3D(-2, 3, -4));

            testMissing("null translation", null, new Vector3D(2, 0, 0, 0), new Vector3D(1, 1, 1));
            testMissing("null rotation", new Vector3D(1, 2, 3), null, new Vector3D(1, 1, 1));
            testMissing("null scale", new Vector3D(1, 2, 3), new Vector3D(2, 0, 0, 0), null);
        }

        private function testQuaternion(label:String, x:Number, y:Number, z:Number, w:Number):void {
            testComponents(label, Vector.<Vector3D>([
                new Vector3D(4, 5, 6),
                new Vector3D(x, y, z, w),
                new Vector3D(2, 3, 4)
            ]));
        }

        private function testScale(label:String, rotation:Vector3D, scale:Vector3D):void {
            testComponents(label, Vector.<Vector3D>([
                new Vector3D(4, 5, 6),
                rotation,
                scale
            ]));
        }

        private function testMissing(
            label:String,
            translation:Vector3D,
            rotation:Vector3D,
            scale:Vector3D
        ):void {
            testComponents(label, Vector.<Vector3D>([translation, rotation, scale]));
        }

        private function testComponents(label:String, components:Vector.<Vector3D>):void {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));
            trace(label + ":");
            try {
                trace("res: " + matrix.recompose(components, Orientation3D.QUATERNION));
            } catch (e:Error) {
                trace("Caught error: " + e.getStackTrace());
            }
            trace("raw: " + matrix.rawData);
        }
    }
}
