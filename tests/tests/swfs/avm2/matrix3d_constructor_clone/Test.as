package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            testConstructors();
            testClone();
        }

        function testConstructors() {
            trace("constructor:");
            testConstructor(null);
            testConstructor([]);
            testConstructor([1]);
            testConstructor([1,2]);
            testConstructor([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15]);
            testConstructor([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
            testConstructor([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17]);
        }

        function testConstructor(v:Array) {
            var matrix:Matrix3D;

            if (v === null) {
                matrix = new Matrix3D(null);
            } else {
                matrix = new Matrix3D(Vector.<Number>(v));
            }

            trace(matrix.rawData);
        }

        function testClone() {
            trace("clone:");
            var matrix:Matrix3D = new Matrix3D();
            trace(matrix.rawData);
            var matrix2:Matrix3D = matrix.clone();
            trace(matrix2.rawData);
            matrix2.rawData = Vector.<Number>([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
            trace(matrix.rawData);
            trace(matrix2.rawData);
            matrix.rawData = Vector.<Number>([16,15,14,13,12,11,10,9,8,7,6,5,4,3,2,1]);
            trace(matrix.rawData);
            trace(matrix2.rawData);
        }
    }
}
