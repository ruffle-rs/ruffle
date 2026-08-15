package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            testExceptions();
            testSimpleValues();
            testSpecialValues();
        }

        function testSimpleValues() {
            var a:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));

            testScale("1,1,1", a, 1, 1, 1);
            testScale("-1,-1,-1", a, -1, -1, -1);
            testScale("-1,1,-1", a, -1, 1, -1);
            testScale("1,1,-1", a, 1, 1, -1);
            testScale("-1,1,1", a, -1, 1, 1);
            testScale("2,3,4", a, 2, 3, 4);
            testScale("-3,7,-1", a, -3, 7, -1);
        }

        function testSpecialValues() {
            var a:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));

            testScale("NaN,1,1", a, NaN, 1, 1);
            testScale("1,NaN,1,1", a, 1, NaN, 1);
            testScale("1,1,NaN,1,1", a, 1, 1, NaN);

            testScale("Infinity,1,1", a, Infinity, 1, 1);
            testScale("1,Infinity,1,1", a, 1, Infinity, 1);
            testScale("1,1,Infinity,1,1", a, 1, 1, Infinity);

            var b:Matrix3D = new Matrix3D(Vector.<Number>([
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
            ]));

            testScale("NaN,1,1", b, NaN, 1, 1);
            testScale("1,NaN,1,1", b, 1, NaN, 1);
            testScale("1,1,NaN,1,1", b, 1, 1, NaN);

            testScale("Infinity,1,1", b, Infinity, 1, 1);
            testScale("1,Infinity,1,1", b, 1, Infinity, 1);
            testScale("1,1,Infinity,1,1", b, 1, 1, Infinity);
        }

        function testScale(label:String, m:Matrix3D, x:Number, y:Number, z:Number) {
            m = m.clone()
            m.appendScale(x,y,z);
            trace(label + " (append): " + m.rawData);

            m = m.clone()
            m.prependScale(x,y,z);
            trace(label + " (prepend): " + m.rawData);
        }

        function testExceptions() {
            testException(function() {
                new Matrix3D().appendScale(0.0, 1.0, 1.0);
            });
            testException(function() {
                new Matrix3D().appendScale(1.0, 0.0, 1.0);
            });
            testException(function() {
                new Matrix3D().appendScale(1.0, 1.0, 0.0);
            });
            testException(function() {
                new Matrix3D().appendScale(0.0, 0.0, 0.0);
            });

            testException(function() {
                new Matrix3D().prependScale(0.0, 1.0, 1.0);
            });
            testException(function() {
                new Matrix3D().prependScale(1.0, 0.0, 1.0);
            });
            testException(function() {
                new Matrix3D().prependScale(1.0, 1.0, 0.0);
            });
            testException(function() {
                new Matrix3D().prependScale(0.0, 0.0, 0.0);
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
