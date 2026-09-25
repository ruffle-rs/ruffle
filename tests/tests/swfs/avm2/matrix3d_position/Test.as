package {
    import flash.display.Sprite;
    import flash.geom.*;

    // All values here are small integers, NaN, or infinities, so nothing
    // needs to be compared with an approximation.
    public class Test extends Sprite {
        public function Test() {
            testGetter();
            testSetter();
        }

        function makeMatrix():Matrix3D {
            return new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));
        }

        function describe(v:Vector3D):String {
            return "(" + v.x + ", " + v.y + ", " + v.z + ", " + v.w + ")";
        }

        function testGetter() {
            testGet("basic", [
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
                13, 14, 15, 16
            ]);
            testGet("NaN x", [
                1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, NaN, 2, 3, 1
            ]);
            testGet("NaN y", [
                1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, NaN, 3, 1
            ]);
            testGet("NaN z", [
                1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 2, NaN, 1
            ]);
            testGet("infinite", [
                1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, Infinity, -Infinity, NaN, 1
            ]);
            testGet("unusual rawData[15]", [
                1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 2, 3, NaN
            ]);
        }

        function testGet(label:String, rawData:Array) {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>(rawData));
            var position:Vector3D = matrix.position;
            trace("get position " + label + ": " + describe(position));
            trace("  matrix: " + matrix.rawData);
        }

        function testSetter() {
            testSet("basic", new Vector3D(101, 102, 103));
            testSet("w set too", new Vector3D(101, 102, 103, 999));
            testSet("NaN x", new Vector3D(NaN, 102, 103));
            testSet("NaN y", new Vector3D(101, NaN, 103));
            testSet("NaN z", new Vector3D(101, 102, NaN));
            testSet("infinite", new Vector3D(Infinity, -Infinity, NaN));
            testSet("null", null);
        }

        function testSet(label:String, val:Vector3D) {
            var matrix:Matrix3D = makeMatrix();
            try {
                matrix.position = val;
                trace("set position " + label + ": " + matrix.rawData);
            } catch (e) {
                trace("set position " + label + " threw: " + e.getStackTrace());
            }
        }
    }
}
