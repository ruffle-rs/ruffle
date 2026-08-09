package {
    import flash.display.Sprite;
    import flash.geom.*;

    // All values here are small integers, NaN, or infinities, so nothing
    // needs to be compared with an approximation.
    public class Test extends Sprite {
        public function Test() {
            testTranspose("distinct entries", [
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
                13, 14, 15, 16
            ]);

            // A symmetric matrix, where transposing shouldn't change anything.
            testTranspose("symmetric", [
                1, 2, 3, 4,
                2, 5, 6, 7,
                3, 6, 8, 9,
                4, 7, 9, 10
            ]);

            // NaN and infinities off the diagonal.
            testTranspose("special off diagonal", [
                1, NaN, 3, Infinity,
                2, 5, -Infinity, 7,
                3, NaN, 8, 9,
                -Infinity, 7, 9, 10
            ]);

            // NaN and infinities on the diagonal, which transpose leaves
            // alone.
            testTranspose("special on diagonal", [
                NaN, 2, 3, 4,
                2, Infinity, 6, 7,
                3, 6, -Infinity, 9,
                4, 7, 9, NaN
            ]);

            testDoubleTranspose([
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
                13, 14, 15, 16
            ]);
        }

        function testTranspose(label:String, rawData:Array) {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>(rawData));
            matrix.transpose();
            trace("transpose " + label + ": " + matrix.rawData);
        }

        function testDoubleTranspose(rawData:Array) {
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>(rawData));
            matrix.transpose();
            matrix.transpose();
            trace("double transpose: " + matrix.rawData);
        }
    }
}
