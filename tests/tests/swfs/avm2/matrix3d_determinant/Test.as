package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            basic();
            specialBasic();
            conditionalLaplaceExpansion();
            rawDataChange();
        }

        function basic() {
            testDeterminant("identity", [
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ]);
            testDeterminant("scale", [
                2, 0, 0, 0,
                0, 3, 0, 0,
                0, 0, 4, 0,
                0, 0, 0, 1
            ]);
            // Not a valid affine matrix: the bottom row isn't (0, 0, 0, 1).
            testDeterminant("diagonal", [
                2, 0, 0, 0,
                0, 3, 0, 0,
                0, 0, 4, 0,
                0, 0, 0, 5
            ]);
            testDeterminant("mirrored x", [
                -1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ]);
            testDeterminant("translation only", [
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                5, 6, 7, 1
            ]);
            testDeterminant("sheared", [
                1, 0, 0, 0,
                3, 4, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ]);
            // Rows in arithmetic progression, so they're linearly dependent.
            testDeterminant("degenerate", [
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
                13, 14, 15, 16
            ]);
            testDeterminant("zero", [
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0
            ]);
            // A general matrix, with no zeroes and no particular structure.
            testDeterminant("dense", [
                1, 2, 1, 2,
                2, 3, 1, 2,
                3, 1, 2, 3,
                1, 2, 3, 3
            ]);
            // Not affine: the bottom row isn't (0, 0, 0, 1).
            testDeterminant("not affine", [
                2, 1, -1, 0,
                -1, 3, -2, 0,
                3, 2, -2, 0,
                0, 1, 1, 1
            ]);
            // Combines a non-affine bottom row with a non-trivial upper-left
            // block and a translation column, so every entry of the matrix
            // takes part.
            testDeterminant("not affine with translation", [
                1, 2, 1, 4,
                2, 3, 2, 1,
                3, 3, 4, 2,
                4, 2, 4, 1
            ]);
        }

        function specialBasic() {
            testSpecialValues("zero", [
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0,
                0, 0, 0, 0
            ]);
            testSpecialValues("identity", [
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ]);
            testSpecialValues("dense", [
                1, 2, 1, 2,
                2, 3, 1, 2,
                3, 1, 2, 3,
                1, 2, 3, 3
            ]);
        }

        function conditionalLaplaceExpansion() {
            var q = Infinity;
            testDeterminant("laplace 0c", [
                1, 0, 0, 0,
                q, 1, 1, 1,
                q, 1, 1, 1,
                q, 1, 1, 1
            ]);
            testDeterminant("laplace 0r", [
                1, q, q, q,
                0, 1, 1, 1,
                0, 1, 1, 1,
                0, 1, 1, 1
            ]);
            testDeterminant("laplace 5c", [
                1, q, 1, 1,
                0, 1, 0, 0,
                1, q, 1, 1,
                1, q, 1, 1
            ]);
            testDeterminant("laplace 5r", [
                1, 0, 1, 1,
                q, 1, q, q,
                1, 0, 1, 1,
                1, 0, 1, 1
            ]);
            testDeterminant("laplace 10c", [
                1, 1, q, 1,
                1, 1, q, 1,
                0, 0, 1, 0,
                1, 1, q, 1
            ]);
            testDeterminant("laplace 10r", [
                1, 1, 0, 1,
                1, 1, 0, 1,
                q, q, 1, q,
                1, 1, 0, 1
            ]);
            testDeterminant("laplace 10c", [
                1, 1, 1, q,
                1, 1, 1, q,
                1, 1, 1, q,
                0, 0, 0, 1
            ]);
            testDeterminant("laplace 10r", [
                1, 1, 1, 0,
                1, 1, 1, 0,
                1, 1, 1, 0,
                q, q, q, 1
            ]);

            testDeterminant("laplace 10r 2", [
                1, 1, 1, 0,
                1, 1, 1, 0,
                1, 1, 1, 0,
                q, q, q, 2
            ]);
            testDeterminant("laplace 10r 0", [
                1, 1, 1, 0,
                1, 1, 1, 0,
                1, 1, 1, 0,
                q, q, q, 0
            ]);

            // Check if the inner 3x3 is calculated at all.
            for (var j = 0; j < 3; ++j) {
                for (var i = 0; i < 3; ++i) {
                    var m = [
                        q, 1, 1, 0,
                        1, 1, 1, 0,
                        1, 1, 1, 0,
                        q, q, q, 0
                    ];
                    var ix = i + j * 4;
                    m[ix] = q;
                    testDeterminant("laplace 10r [" + ix + "]=inf", [
                        q, 1, 1, 0,
                        1, 1, 1, 0,
                        1, 1, 1, 0,
                        q, q, q, 0
                    ]);
                }
            }

            // Check if a similar logic exists for the inner 3x3.
            testDeterminant("inner laplace 0c", [
                1, 0, 0, 0,
                q, 1, 1, 0,
                q, 1, 1, 0,
                q, q, q, 1
            ]);
            testDeterminant("inner laplace 0r", [
                1, q, q, 0,
                0, 1, 1, 0,
                0, 1, 1, 0,
                q, q, q, 1
            ]);
            testDeterminant("inner laplace 5c", [
                1, q, 1, 0,
                0, 1, 0, 0,
                1, q, 1, 0,
                q, q, q, 1
            ]);
            testDeterminant("inner laplace 5r", [
                1, 0, 1, 0,
                q, 1, q, 0,
                1, 0, 1, 0,
                q, q, q, 1
            ]);
            testDeterminant("inner laplace 10c", [
                1, 1, q, 0,
                1, 1, q, 0,
                0, 0, 1, 0,
                q, q, q, 1
            ]);
            testDeterminant("inner laplace 10r", [
                1, 1, 0, 0,
                1, 1, 0, 0,
                q, q, 1, 0,
                q, q, q, 1
            ]);
        }

        function rawDataChange() {
            testRawDataChange("projection, then none", [
                1, 0, 0, 1,
                0, 1, 0, 0,
                0, 0, 1, 0,
                Infinity, 0, 0, 1
            ], [
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                Infinity, 0, 0, 1
            ]);
            testRawDataChange("none, then projection", [
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                Infinity, 0, 0, 1
            ], [
                1, 0, 0, 1,
                0, 1, 0, 0,
                0, 0, 1, 0,
                Infinity, 0, 0, 1
            ]);
        }

        function testSpecialValues(label:String, base:Array) {
            for each (var special in [NaN, Infinity, -Infinity]) {
                for (var i:int = 0; i < 16; i++) {
                    var rawData:Array = base.concat();
                    rawData[i] = special;
                    testDeterminant(label + " index " + i + " = " + special, rawData);
                }
            }
        }

        function testRawDataChange(label:String, before:Array, after:Array) {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>(before));
            var determinant:Number = matrix.determinant;

            matrix.rawData = Vector.<Number>(after);
            trace(label + ": " + determinant + ", " + matrix.determinant);
        }

        function testDeterminant(label:String, rawData:Array) {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>(rawData));
            trace(label + ": " + matrix.determinant);
        }
    }
}
