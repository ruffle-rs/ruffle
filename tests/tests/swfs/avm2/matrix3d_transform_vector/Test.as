package {
    import flash.display.Sprite;
    import flash.geom.*;

    // All values here are small integers, NaN, or infinities, so nothing
    // needs to be compared with an approximation.
    public class Test extends Sprite {
        public function Test() {
            testDeltaTransformVector();
            testTransformVector();
            testTransformVectors();
        }

        function makeMatrix():Matrix3D {
            return new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));
        }

        function describe(v:Vector3D):String {
            return "(" + v.x + ", " + v.y + ", " + v.z + ", " + v.w + ")";
        }

        function testDeltaTransformVector() {
            testDelta("basic", new Vector3D(1, 2, 3));
            testDelta("w set", new Vector3D(1, 2, 3, 999));
            testDelta("NaN x", new Vector3D(NaN, 2, 3));
            testDelta("NaN y", new Vector3D(1, NaN, 3));
            testDelta("NaN z", new Vector3D(1, 2, NaN));
            testDelta("infinite", new Vector3D(Infinity, -Infinity, NaN));
            testDelta("null", null);
        }

        function testDelta(label:String, v:Vector3D) {
            var matrix:Matrix3D = makeMatrix();
            try {
                var result:Vector3D = matrix.deltaTransformVector(v);
                trace("deltaTransformVector " + label + ": " + describe(result));
            } catch (e) {
                trace("deltaTransformVector " + label + " threw: " + e.getStackTrace());
            }
        }

        function testTransformVector() {
            testTransform("basic", new Vector3D(1, 2, 3));
            testTransform("w set", new Vector3D(1, 2, 3, 999));
            testTransform("NaN x", new Vector3D(NaN, 2, 3));
            testTransform("NaN y", new Vector3D(1, NaN, 3));
            testTransform("NaN z", new Vector3D(1, 2, NaN));
            testTransform("infinite", new Vector3D(Infinity, -Infinity, NaN));
            testTransform("null", null);
        }

        function testTransform(label:String, v:Vector3D) {
            var matrix:Matrix3D = makeMatrix();
            try {
                var result:Vector3D = matrix.transformVector(v);
                trace("transformVector " + label + ": " + describe(result));
            } catch (e) {
                trace("transformVector " + label + " threw: " + e.getStackTrace());
            }
        }

        function testTransformVectors() {
            testVectors("vin length is a multiple of 3", [1, 2, 3, 4, 5, 6], [0, 0, 0, 0, 0, 0], false);
            testVectors("vin length is not a multiple of 3", [1, 2, 3, 4], [0, 0, 0], false);
            testVectors("empty vout", [1, 2, 3, 4, 5, 6], [], false);
            testVectors("vout longer than vin", [1, 2, 3], [101, 102, 103, 104, 105], false);
            testVectors("fixed vout, same length as vin", [1, 2, 3, 4, 5, 6], [0, 0, 0, 0, 0, 0], true);
            testVectors("fixed vout, longer than vin", [1, 2, 3], [0, 0, 0, 0, 0, 0], true);
            testVectors("fixed vout, shorter than vin", [1, 2, 3, 4, 5, 6], [0, 0, 0], true);
            testVectors("empty vin", [], [101, 102, 103], false);

            var special:Vector.<Number> = Vector.<Number>([NaN, Infinity, -Infinity]);
            testVectorsWith("special values", special, Vector.<Number>([0, 0, 0]));

            var same:Vector.<Number> = Vector.<Number>([1, 2, 3, 4, 5, 6]);
            testVectorsWith("vin and vout are the same object", same, same);

            testVectorsNull();
        }

        function testVectors(label:String, vinData:Array, voutData:Array, fixed:Boolean) {
            var vin:Vector.<Number> = Vector.<Number>(vinData);
            var vout:Vector.<Number> = Vector.<Number>(voutData);
            vout.fixed = fixed;
            testVectorsWith(label, vin, vout);
        }

        function testVectorsWith(label:String, vin:Vector.<Number>, vout:Vector.<Number>) {
            var matrix:Matrix3D = makeMatrix();
            try {
                matrix.transformVectors(vin, vout);
                trace("transformVectors " + label + ": vout = " + vout
                    + ", fixed = " + vout.fixed + ", length = " + vout.length);
            } catch (e) {
                trace("transformVectors " + label + " threw: " + e.getStackTrace());
            }
        }

        function testVectorsNull() {
            var matrix:Matrix3D = makeMatrix();
            var vin:Vector.<Number> = Vector.<Number>([1, 2, 3]);
            var vout:Vector.<Number> = Vector.<Number>([0, 0, 0]);

            try {
                matrix.transformVectors(null, vout);
                trace("transformVectors null vin: didn't throw");
            } catch (e) {
                trace("transformVectors null vin threw: " + e.getStackTrace());
            }

            try {
                matrix.transformVectors(vin, null);
                trace("transformVectors null vout: didn't throw");
            } catch (e) {
                trace("transformVectors null vout threw: " + e.getStackTrace());
            }

            try {
                matrix.transformVectors(null, null);
                trace("transformVectors null both: didn't throw");
            } catch (e) {
                trace("transformVectors null both threw: " + e.getStackTrace());
            }
        }
    }
}
