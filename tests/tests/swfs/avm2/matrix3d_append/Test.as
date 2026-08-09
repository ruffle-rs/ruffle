package {
    import flash.display.Sprite;
    import flash.geom.*;

    // All matrices here are made of small integers, so that the products
    // 'append' computes are exactly representable, without needing to
    // approximate anything.
    public class Test extends Sprite {
        public function Test() {
            testExceptions();
            testIdentity();
            testSimpleValues();
            testSelfAppend();
            testSpecialValues();
        }

        // Appending the identity matrix on either side should leave the
        // other matrix untouched.
        function testIdentity() {
            var identity:Matrix3D = new Matrix3D();
            var values:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));

            var a:Matrix3D = values.clone();
            a.append(identity);
            trace("values.append(identity): " + a.rawData);

            var b:Matrix3D = identity.clone();
            b.append(values);
            trace("identity.append(values): " + b.rawData);
        }

        // Two matrices with no particular structure, to check the general
        // case of the multiplication.
        function testSimpleValues() {
            var a:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));
            var b:Matrix3D = new Matrix3D(Vector.<Number>([
                16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1
            ]));

            var ab:Matrix3D = a.clone();
            ab.append(b);
            trace("a.append(b): " + ab.rawData);

            var ba:Matrix3D = b.clone();
            ba.append(a);
            trace("b.append(a): " + ba.rawData);
        }

        // A matrix appended to itself, to check what happens when 'lhs' and
        // 'this' are the same object and its data can't be read twice
        // without a copy.
        function testSelfAppend() {
            var a:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));
            a.append(a);
            trace("a.append(a): " + a.rawData);
        }

        // NaN and infinities in either matrix.
        function testSpecialValues() {
            testAppend("NaN lhs", [
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                1, 2, 3, 1
            ], [
                NaN, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ]);
            testAppend("NaN this", [
                NaN, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ], [
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                1, 2, 3, 1
            ]);
            testAppend("infinite lhs", [
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                1, 2, 3, 1
            ], [
                Infinity, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ]);
            testAppend("infinite translation", [
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                Infinity, -Infinity, NaN, 1
            ], [
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                1, 2, 3, 1
            ]);
            // Infinity times zero, off the diagonal.
            testAppend("infinity times zero", [
                Infinity, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ], [
                0, 1, 0, 0,
                1, 0, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ]);
        }

        function testAppend(label:String, thisData:Array, lhsData:Array) {
            var a:Matrix3D = new Matrix3D(Vector.<Number>(thisData));
            var b:Matrix3D = new Matrix3D(Vector.<Number>(lhsData));
            a.append(b);
            trace(label + ": " + a.rawData);
        }

        function testExceptions() {
            testException(function() {
                new Matrix3D().append(null);
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
