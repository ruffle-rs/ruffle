package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            testExceptions();
            testRows();
            testSpecialValuesTo();
            testSpecialValuesFrom();
            testRowCoercion();
        }

        function makeMatrix():Matrix3D {
            return new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));
        }

        function describe(v:Vector3D):String {
            return "(" + v.x + ", " + v.y + ", " + v.z + ", " + v.w + ")";
        }

        // Reads each row back out, to check both the mapping from row
        // to 'rawData' indices, and that the matrix itself is left alone.
        function testRows() {
            for each (var row in [0, 1, 2, 3]) {
                var matrix:Matrix3D = makeMatrix();
                var v:Vector3D = new Vector3D(-1, -2, -3, -4);
                matrix.copyRowTo(row, v);
                trace("copyRowTo(" + row + "): " + describe(v)
                    + ", matrix " + matrix.rawData);
            }

            for each (row in [0, 1, 2, 3]) {
                var m:Matrix3D = makeMatrix();
                m.copyRowFrom(row, new Vector3D(101, 102, 103, 104));
                trace("copyRowFrom(" + row + "): " + m.rawData);
            }
        }

        // NaN and infinities already in the matrix, read out with
        // copyRowTo.
        function testSpecialValuesTo() {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>([
                NaN, Infinity, -Infinity, 0,
                1, NaN, 2, 3,
                Infinity, -Infinity, NaN, Infinity,
                -Infinity, 0, 0, NaN
            ]));

            for each (var row in [0, 1, 2, 3]) {
                var v:Vector3D = new Vector3D();
                matrix.copyRowTo(row, v);
                trace("copyRowTo(" + row + ") special: " + describe(v));
            }
        }

        // NaN and infinities in the Vector3D, written with copyRowFrom.
        function testSpecialValuesFrom() {
            for each (var row in [0, 1, 2, 3]) {
                var matrix:Matrix3D = makeMatrix();
                matrix.copyRowFrom(row, new Vector3D(NaN, Infinity, -Infinity, NaN));
                trace("copyRowFrom(" + row + ") special: " + matrix.rawData);
            }
        }

        // How the 'row' argument is coerced to uint.
        function testRowCoercion() {
            for each (var row in [
                4, 5, 0xFFFFFFFF, 4294967296 + 4,
                2.9, 3.9, -1, -0.5,
                NaN, Infinity, -Infinity,
                true, false, "2", "3.9", "-1", null, undefined
            ]) {
                var matrix:Matrix3D = makeMatrix();
                var v:Vector3D = new Vector3D(-1, -2, -3, -4);
                try {
                    matrix.copyRowTo(row, v);
                    trace("copyRowTo(" + row + ") coerced: " + describe(v));
                } catch (e) {
                    trace("copyRowTo(" + row + ") coerced: " + e);
                }
            }
        }

        function testExceptions() {
            // Row out of range.
            testException(function() {
                makeMatrix().copyRowTo(4, new Vector3D());
            });
            testException(function() {
                makeMatrix().copyRowFrom(4, new Vector3D());
            });
            testException(function() {
                makeMatrix().copyRowTo(0xFFFFFFFF, new Vector3D());
            });
            testException(function() {
                makeMatrix().copyRowFrom(0xFFFFFFFF, new Vector3D());
            });

            // A null Vector3D with a valid row.
            testException(function() {
                makeMatrix().copyRowTo(0, null);
            });
            testException(function() {
                makeMatrix().copyRowFrom(0, null);
            });

            // A null Vector3D and an out of range row at once.
            testException(function() {
                makeMatrix().copyRowTo(4, null);
            });
            testException(function() {
                makeMatrix().copyRowFrom(4, null);
            });

            // The matrix must be left untouched when 'copyRowFrom' throws.
            var matrix:Matrix3D = makeMatrix();
            try {
                matrix.copyRowFrom(4, new Vector3D(101, 102, 103, 104));
            } catch (e) {}
            trace("Untouched after error: " + matrix.rawData);
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
