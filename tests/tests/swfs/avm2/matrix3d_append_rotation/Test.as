package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            testExceptions();
            testDegreesZero();
            testAxisZero();
            testDegreesSpecial();
            testPivotPoint();
            testIntegration();
        }

        function identity():Matrix3D {
            return new Matrix3D();
        }

        // 0 degrees, various axes.
        function testDegreesZero() {
            testAppendRotation("0 degrees, x axis", 0, new Vector3D(1, 0, 0));
            testAppendRotation("0 degrees, y axis", 0, new Vector3D(0, 1, 0));
            testAppendRotation("0 degrees, z axis", 0, new Vector3D(0, 0, 1));
            testAppendRotation("0 degrees, non-unit axis", 0, new Vector3D(3, 4, 0));
            testAppendRotation("0 degrees, NaN axis", 0, new Vector3D(NaN, 0, 0));
            testAppendRotation("0 degrees, infinite axis", 0, new Vector3D(Infinity, 0, 0));
            testAppendRotation("-0 degrees, x axis", -0, new Vector3D(1, 0, 0));
        }

        // The zero vector as an axis, various finite degrees.
        function testAxisZero() {
            testAppendRotation("zero axis, 0 degrees", 0, new Vector3D(0, 0, 0));
            testAppendRotation("zero axis, 180 degrees", 180, new Vector3D(0, 0, 0));
        }

        // Degrees that aren't an ordinary finite number.
        function testDegreesSpecial() {
            testAppendRotation("NaN degrees, unit axis", NaN, new Vector3D(1, 0, 0));
            testAppendRotation("NaN degrees, zero axis", NaN, new Vector3D(0, 0, 0));
            testAppendRotation("infinite degrees, unit axis", Infinity, new Vector3D(1, 0, 0));
            testAppendRotation("infinite degrees, zero axis", Infinity, new Vector3D(0, 0, 0));
        }

        // 'pivotPoint', under conditions where 'degrees' and 'axis' are
        // exactly representable on their own.
        function testPivotPoint() {
            testAppendRotation("0 degrees, pivot set", 0, new Vector3D(1, 2, 3), new Vector3D(10, 20, 30));
            testAppendRotation("0 degrees, pivot explicitly null", 0, new Vector3D(1, 2, 3), null);
            testAppendRotation("zero axis, NaN pivot", 90, new Vector3D(0, 0, 0), new Vector3D(NaN, NaN, NaN));
        }

        // The same cases, appended onto a matrix with distinct entries,
        // to check the multiplication in 'append' this calls into.
        function testIntegration() {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));
            matrix.appendRotation(0, new Vector3D(1, 0, 0));
            trace("integration, 0 degrees: " + matrix.rawData);
        }

        function testAppendRotation(label:String, degrees:Number, axis:Vector3D, pivotPoint:Vector3D = null) {
            var matrix:Matrix3D = identity();
            try {
                matrix.appendRotation(degrees, axis, pivotPoint);
                trace("appendRotation " + label + ": " + matrix.rawData);
            } catch (e) {
                trace("appendRotation " + label + " threw: " + e.getStackTrace());
            }
        }

        function testExceptions() {
            testException(function() {
                identity().appendRotation(0, null);
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
