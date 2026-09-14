package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            testGetSet();
            testComponents();
            testGetIsCopy();
            testSetIsCopy();
            testSetNull();
            testSetLengths();
            testSpecialValues();
            testPrecision();
        }

        function makeMatrix():Matrix3D {
            return new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));
        }

        // Basic round trip through the getter and setter.
        function testGetSet() {
            var matrix:Matrix3D = makeMatrix();
            trace("rawData: " + matrix.rawData);

            var other:Matrix3D = new Matrix3D();
            other.rawData = matrix.rawData;
            trace("other.rawData: " + other.rawData);
        }

        // Check if every component is settable independent of others.
        function testComponents() {
            for (var i = 0; i < 15; ++i) {
                var matrix:Matrix3D = makeMatrix();
                var rawData = matrix.rawData;
                rawData[i] = 100;

                matrix.rawData = rawData;
                trace("After setting " + i + "th: " + matrix.rawData);
            }
        }

        // Calling the getter twice, and mutating one of the results.
        function testGetIsCopy() {
            var matrix:Matrix3D = makeMatrix();
            var a:Vector.<Number> = matrix.rawData;
            var b:Vector.<Number> = matrix.rawData;
            trace("Same instance: " + (a == b));

            a[0] = 1000;
            trace("Mutated copy: " + a);
            trace("Other copy: " + b);
            trace("Matrix after mutating copy: " + matrix.rawData);
        }

        // Mutating the source Vector after passing it to the setter.
        function testSetIsCopy() {
            var source:Vector.<Number> = Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]);
            var matrix:Matrix3D = new Matrix3D();
            matrix.rawData = source;
            source[0] = 1000;
            trace("Matrix after mutating source: " + matrix.rawData);
        }

        // Setting rawData to null.
        function testSetNull() {
            var matrix:Matrix3D = makeMatrix();
            matrix.rawData = null;
            trace("After rawData = null: " + matrix.rawData);
        }

        // Setting rawData to Vectors of lengths other than 16.
        function testSetLengths() {
            for each (var length in [0, 1, 15, 16, 17, 32]) {
                var matrix:Matrix3D = makeMatrix();
                var data:Vector.<Number> = new Vector.<Number>();
                for (var i:int = 0; i < length; i++) {
                    data.push(i + 100);
                }
                matrix.rawData = data;
                trace("Length " + length + ": " + matrix.rawData);
            }
        }

        // NaN and infinities round tripped through the setter and getter.
        function testSpecialValues() {
            var matrix:Matrix3D = new Matrix3D();
            matrix.rawData = Vector.<Number>([
                NaN, Infinity, -Infinity, 0,
                1, NaN, 2, 3,
                Infinity, -Infinity, NaN, Infinity,
                -Infinity, 0, 0, NaN
            ]);
            trace("Special values: " + matrix.rawData);
        }

        function testPrecision() {
            var values = [
                1.0000001,
                // Represented as 1 in 32-bit precision.
                1.00000001,
                // Represented as +inf in 32-bit precision.
                1.0e50
            ];
            for each (var value in values) {
                var matrix:Matrix3D = new Matrix3D();
                matrix.rawData = Vector.<Number>([
                    value,value,value,value,
                    value,value,value,value,
                    value,value,value,value,
                    value,value,value,value
                ]);
                trace("Precision (" + value + "): " + matrix.rawData);
            }
        }
    }
}
