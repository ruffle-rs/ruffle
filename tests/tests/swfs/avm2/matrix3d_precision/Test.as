package {
    import flash.display.Sprite;
    import flash.geom.*;

    // At this point we know Flash stores matrix components as 32-bit floats,
    // the question is what precision is being used for operations, as at some
    // point we have to switch from 32-bit to 64-bit float (atoms).
    public class Test extends Sprite {
        public function Test() {
            testAppendTranslation();
            testPrependTranslation();
            testAppendScale();
            testPrependScale();
            testCopyFrom();
            testPosition();
            testDeterminant();
            testTransformVectors();
        }

        function testAppendTranslation() {
            // In appendTranslation() we are adding f32 to f64, and the place
            // where the cast happens is observable.

            var matrix:Matrix3D = newMatrix(1.1920928955078125e-7);
            var f = 1.0000000596046448;
            matrix.appendTranslation(f, f, f);

            trace("testAppendTranslation: " + matrix.rawData[12] +
                "," + matrix.rawData[13] +
                "," + matrix.rawData[14]);
        }

        function testPrependTranslation() {
            // Unlike appendTranslation(), this one is a full matrix multiplication,
            // so the sum of the four component-wise products has to land somewhere.

            var matrix:Matrix3D = newMatrix(3);
            var f = 1.0000001;
            matrix.prependTranslation(f, f, f);

            trace("testPrependTranslation: " + matrix.rawData[12] +
                "," + matrix.rawData[13] +
                "," + matrix.rawData[14]);
        }

        function testAppendScale() {
            var matrix:Matrix3D = newMatrix(3);
            var f = 1.0000000596046448;
            matrix.appendScale(f, f, f);

            trace("testAppendScale: " + matrix.rawData[0] +
                "," + matrix.rawData[5] +
                "," + matrix.rawData[10]);
        }

        function testPrependScale() {
            var matrix:Matrix3D = newMatrix(3);
            var f = 1.0000000596046448;
            matrix.prependScale(f, f, f);

            trace("testPrependScale: " + matrix.rawData[0] +
                "," + matrix.rawData[5] +
                "," + matrix.rawData[10]);
        }

        function testCopyFrom() {
            // Just to make sure nothing funky is going on here.

            var matrix:Matrix3D = newMatrix(1);

            matrix.copyColumnFrom(0, new Vector3D(1.0e50, 1.0e50, 1.0e50));
            matrix.copyColumnFrom(1, new Vector3D(1.00000001, 1.00000001, 1.00000001));
            trace("copyColumnFrom: " + matrix.rawData[0] + "," + matrix.rawData[4]);

            matrix.copyRowFrom(0, new Vector3D(1.0e50, 1.0e50, 1.0e50));
            matrix.copyRowFrom(1, new Vector3D(1.00000001, 1.00000001, 1.00000001));
            trace("copyRowFrom: " + matrix.rawData[0] + "," + matrix.rawData[1]);

            matrix.copyRawDataFrom(Vector.<Number>([
                1.0e50,1.00000001,0,0,
                0,0,0,0,
                0,0,0,0,
                0,0,0,0
            ]));
            trace("copyRawDataFrom: " + matrix.rawData[0] + "," + matrix.rawData[1]);
        }

        function testPosition() {
            // Just to make sure nothing funky is going on here.

            var matrix:Matrix3D = newMatrix(1);
            matrix.position = new Vector3D(1.0e50, 1.0e50, 1.0e50);
            trace("position: " + matrix.rawData[12] + "," + matrix.rawData[13] + "," + matrix.rawData[14]);
            trace("position: " + matrix.position);
        }

        function testDeterminant() {
            // Let's get a matrix with f32 components that has a determinant not
            // representable as f32.

            var m:Matrix3D = new Matrix3D(Vector.<Number>([
                1e10,0,0,0,
                0,1e10,0,0,
                0,0,1e10,0,
                0,0,0,1e10
            ]));
            trace("determinant: " + m.determinant);

            // Let's now check the Laplace expansion.

            m = new Matrix3D(Vector.<Number>([
                0,    10,20,10,
                1e37, 20,30,10,
                0,    30,10,20,
                1e37, 10,20,30
            ]));
            trace("determinant: " + m.determinant);

            m = new Matrix3D(Vector.<Number>([
                1e37, 10,20,10,
                0,    20,30,10,
                1e37,30,10,20,
                0,    10,20,30
            ]));
            trace("determinant: " + m.determinant);
        }

        function testTransformVectors() {
            var matrix1:Matrix3D = newMatrix(1);
            var matrix8:Matrix3D = newMatrix(8);

            testTransformVector(matrix1, new Vector3D(1.0e50, 1.0e50, 1.0e50));
            testTransformVector(matrix1, new Vector3D(1.00000001, 1.00000001, 1.00000001));

            var f1 = 4e37;
            var f2 = 5e37;
            // f1 * 8 is f32
            // f2 * 8 overflows
            testTransformVector(matrix8, new Vector3D(f1, f1, f1));
            testTransformVector(matrix8, new Vector3D(f2, f2, f2));
        }

        function testTransformVector(matrix:Matrix3D, vector:Vector3D) {
            var vin:Vector.<Number> = Vector.<Number>([vector.x, vector.y, vector.z]);
            var vout:Vector.<Number> = new Vector.<Number>();
            matrix.transformVectors(vin, vout);

            trace("testTransformVector (" + matrix.rawData + "," + vector + "):");
            trace("  transformVector: " + matrix.transformVector(vector));
            trace("  transformVectors: " + vout);
            trace("  deltaTransformVector: " + matrix.deltaTransformVector(vector));
        }

        function newMatrix(v:Number) {
            return new Matrix3D(Vector.<Number>([
                v,v,v,v,
                v,v,v,v,
                v,v,v,v,
                v,v,v,v
            ]));
        }
    }
}
