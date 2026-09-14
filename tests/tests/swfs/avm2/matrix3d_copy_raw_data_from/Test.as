package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            testExceptions();
            testBasic();
            testTranspose();
            testSpecialValues();
            testDefaults();
        }

        function makeMatrix():Matrix3D {
            return new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));
        }

        function testCopy(label:String, vectorData:Array, index:*, doTranspose:Boolean = false) {
            var matrix:Matrix3D = makeMatrix();
            try {
                if (index === undefined) {
                    matrix.copyRawDataFrom(Vector.<Number>(vectorData));
                } else {
                    matrix.copyRawDataFrom(Vector.<Number>(vectorData), index, doTranspose);
                }
                trace("copyRawDataFrom " + label + ": " + matrix.rawData);
            } catch (e) {
                trace("copyRawDataFrom " + label + " threw: " + e.getStackTrace());
            }
        }

        function testBasic() {
            testCopy("0 entries", [], 0);
            testCopy("3 entries", [201, 202, 203], 0);
            testCopy("15 entries", [
                101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115
            ], 0);
            testCopy("16 entries", [
                101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116
            ], 0);
            testCopy("20 entries", [
                501, 502, 503, 504, 505, 506, 507, 508, 509, 510,
                511, 512, 513, 514, 515, 516, 517, 518, 519, 520
            ], 0);

            testCopy("3 entries offset 1", [301, 302, 303], 1);
            testCopy("3 entries offset 13", [301, 302, 303], 13);
            testCopy("0 entries offset 16", [], 16);

            testCopy("16 entries offset 1", [
                101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116
            ], 1);
            testCopy("17 entries offset 1", [
                101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117
            ], 1);
            testCopy("20 entries offset 4", [
                501, 502, 503, 504, 505, 506, 507, 508, 509, 510,
                511, 512, 513, 514, 515, 516, 517, 518, 519, 520
            ], 4);
            testCopy("20 entries offset 5", [
                501, 502, 503, 504, 505, 506, 507, 508, 509, 510,
                511, 512, 513, 514, 515, 516, 517, 518, 519, 520
            ], 5);
        }

        function testTranspose() {
            testCopy("16 entries, transposed", [
                701, 702, 703, 704, 705, 706, 707, 708, 709, 710, 711, 712, 713, 714, 715, 716
            ], 0, true);
            testCopy("20 entries with offset, transposed", [
                701, 702, 703, 704, 705, 706, 707, 708, 709, 710,
                711, 712, 713, 714, 715, 716, 717, 718, 719, 720
            ], 0, true);
        }

        function testSpecialValues() {
            testCopy("NaN", [
                NaN, NaN, NaN, NaN,
                NaN, NaN, NaN, NaN,
                NaN, NaN, NaN, NaN,
                NaN, NaN, NaN, NaN
            ], 0);
            testCopy("Infinity", [
                Infinity, Infinity, Infinity, Infinity,
                Infinity, Infinity, Infinity, Infinity,
                Infinity, Infinity, Infinity, Infinity,
                Infinity, Infinity, Infinity, Infinity
            ], 0);
        }

        // 'index' and 'transpose' both left at their default values.
        function testDefaults() {
            testCopy("defaults", [
                101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116
            ], undefined);
        }

        function testExceptions() {
            testException(function() {
                makeMatrix().copyRawDataFrom(null);
            });
        }

        function testException(f:Function) {
            try {
                f();
                trace("Didn't throw");
            } catch (e) {
                trace("Caught error: " + e.getStackTrace());
            }
        }
    }
}
