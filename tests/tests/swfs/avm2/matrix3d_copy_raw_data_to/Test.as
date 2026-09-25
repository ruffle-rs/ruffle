package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            testExceptions();
            testBasic();
            testFixed();
            testTranspose();
            testSpecialValues();
            testDefaults();
        }

        function makeMatrix(values:Array = null):Matrix3D {
            if (values == null) {
                values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
            }
            return new Matrix3D(Vector.<Number>(values));
        }

        function makeVector(length:int, fixed:Boolean):Vector.<Number> {
            var vector:Vector.<Number> = new Vector.<Number>(length, fixed);
            for (var i:int = 0; i < length; i++) {
                vector[i] = -1;
            }
            return vector;
        }

        function testCopy(label:String, vectorLength:int, fixed:Boolean, index:*, doTranspose:Boolean = false, matrix:Matrix3D = null) {
            if (matrix == null) {
                matrix = makeMatrix();
            }
            var vector:Vector.<Number> = makeVector(vectorLength, fixed);
            try {
                if (index === undefined) {
                    matrix.copyRawDataTo(vector);
                } else {
                    matrix.copyRawDataTo(vector, index, doTranspose);
                }
                trace("copyRawDataTo " + label + ": " + vector);
            } catch (e) {
                trace("copyRawDataTo " + label + " threw: " + e.getStackTrace());
                trace("copyRawDataTo " + label + " vector after throw: " + vector + ";");
            }
        }

        function testBasic() {
            testCopy("0 length, growable", 0, false, 0);
            testCopy("16 length, growable", 16, false, 0);
            testCopy("20 length, growable", 20, false, 0);

            testCopy("16 length offset 1, growable", 16, false, 1);
            testCopy("17 length offset 1, growable", 17, false, 1);
            testCopy("20 length offset 4, growable", 20, false, 4);
        }

        function testFixed() {
            testCopy("0 length, fixed", 0, true, 0);
            testCopy("16 length, fixed", 16, true, 0);
            testCopy("15 length, fixed", 15, true, 0);
            testCopy("20 length, fixed", 20, true, 0);

            testCopy("16 length offset 1, fixed", 16, true, 1);
            testCopy("17 length offset 1, fixed", 17, true, 1);
        }

        function testTranspose() {
            var matrix:Matrix3D = makeMatrix([
                701, 702, 703, 704, 705, 706, 707, 708, 709, 710, 711, 712, 713, 714, 715, 716
            ]);
            testCopy("16 length, transposed", 16, false, 0, true, matrix);
            testCopy("20 length offset 2, transposed", 20, false, 2, true, matrix);
        }

        function testSpecialValues() {
            var nanMatrix:Matrix3D = makeMatrix([
                NaN, NaN, NaN, NaN,
                NaN, NaN, NaN, NaN,
                NaN, NaN, NaN, NaN,
                NaN, NaN, NaN, NaN
            ]);
            testCopy("NaN", 16, false, 0, false, nanMatrix);

            var infMatrix:Matrix3D = makeMatrix([
                Infinity, Infinity, Infinity, Infinity,
                Infinity, Infinity, Infinity, Infinity,
                Infinity, Infinity, Infinity, Infinity,
                Infinity, Infinity, Infinity, Infinity
            ]);
            testCopy("Infinity", 16, false, 0, false, infMatrix);
        }

        // 'index' and 'transpose' both left at their default values.
        function testDefaults() {
            testCopy("defaults", 16, false, undefined);
        }

        function testExceptions() {
            testException(function() {
                makeMatrix().copyRawDataTo(null);
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
