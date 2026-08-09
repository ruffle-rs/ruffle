package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            testExceptions();
            testColumns();
            testSpecialValuesTo();
            testSpecialValuesFrom();
            testColumnCoercion();
        }

        function makeMatrix():Matrix3D {
            return new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));
        }

        function describe(v:Vector3D):String {
            return "(" + v.x + ", " + v.y + ", " + v.z + ", " + v.w + ")";
        }

        // Reads each column back out, to check both the mapping from column
        // to 'rawData' indices, and that the matrix itself is left alone.
        function testColumns() {
            for each (var column in [0, 1, 2, 3]) {
                var matrix:Matrix3D = makeMatrix();
                var v:Vector3D = new Vector3D(-1, -2, -3, -4);
                matrix.copyColumnTo(column, v);
                trace("copyColumnTo(" + column + "): " + describe(v)
                    + ", matrix " + matrix.rawData);
            }

            for each (column in [0, 1, 2, 3]) {
                var m:Matrix3D = makeMatrix();
                m.copyColumnFrom(column, new Vector3D(101, 102, 103, 104));
                trace("copyColumnFrom(" + column + "): " + m.rawData);
            }
        }

        // NaN and infinities already in the matrix, read out with
        // copyColumnTo.
        function testSpecialValuesTo() {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>([
                NaN, Infinity, -Infinity, 0,
                1, NaN, 2, 3,
                Infinity, -Infinity, NaN, Infinity,
                -Infinity, 0, 0, NaN
            ]));

            for each (var column in [0, 1, 2, 3]) {
                var v:Vector3D = new Vector3D();
                matrix.copyColumnTo(column, v);
                trace("copyColumnTo(" + column + ") special: " + describe(v));
            }
        }

        // NaN and infinities in the Vector3D, written with copyColumnFrom.
        function testSpecialValuesFrom() {
            for each (var column in [0, 1, 2, 3]) {
                var matrix:Matrix3D = makeMatrix();
                matrix.copyColumnFrom(column, new Vector3D(NaN, Infinity, -Infinity, NaN));
                trace("copyColumnFrom(" + column + ") special: " + matrix.rawData);
            }
        }

        // How the 'column' argument is coerced to uint.
        function testColumnCoercion() {
            for each (var column in [
                4, 5, 0xFFFFFFFF, 4294967296 + 4,
                2.9, 3.9, -1, -0.5,
                NaN, Infinity, -Infinity,
                true, false, "2", "3.9", "-1", null, undefined
            ]) {
                var matrix:Matrix3D = makeMatrix();
                var v:Vector3D = new Vector3D(-1, -2, -3, -4);
                try {
                    matrix.copyColumnTo(column, v);
                    trace("copyColumnTo(" + column + ") coerced: " + describe(v));
                } catch (e) {
                    trace("copyColumnTo(" + column + ") coerced: " + e);
                }
            }
        }

        function testExceptions() {
            // Column out of range.
            testException(function() {
                makeMatrix().copyColumnTo(4, new Vector3D());
            });
            testException(function() {
                makeMatrix().copyColumnFrom(4, new Vector3D());
            });
            testException(function() {
                makeMatrix().copyColumnTo(0xFFFFFFFF, new Vector3D());
            });
            testException(function() {
                makeMatrix().copyColumnFrom(0xFFFFFFFF, new Vector3D());
            });

            // A null Vector3D with a valid column.
            testException(function() {
                makeMatrix().copyColumnTo(0, null);
            });
            testException(function() {
                makeMatrix().copyColumnFrom(0, null);
            });

            // A null Vector3D and an out of range column at once.
            testException(function() {
                makeMatrix().copyColumnTo(4, null);
            });
            testException(function() {
                makeMatrix().copyColumnFrom(4, null);
            });

            // The matrix must be left untouched when 'copyColumnFrom' throws.
            var matrix:Matrix3D = makeMatrix();
            try {
                matrix.copyColumnFrom(4, new Vector3D(101, 102, 103, 104));
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
