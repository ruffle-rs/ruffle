package {
    import flash.display.Sprite;
    import flash.geom.*;

    public class Test extends Sprite {
        public function Test() {
            testSimpleValues();
            testSpecialValues();
        }

        function testSimpleValues() {
            var a:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));

            testTranslation("null,null,null", a, null, null, null);
            testTranslation("0,0,0", a, 0, 0, 0);
            testTranslation("1,1,1", a, 1, 1, 1);
            testTranslation("-1,-1,-1", a, -1, -1, -1);
            testTranslation("-1,1,-1", a, -1, 1, -1);
            testTranslation("1,1,-1", a, 1, 1, -1);
            testTranslation("-1,1,1", a, -1, 1, 1);
            testTranslation("2,3,4", a, 2, 3, 4);
            testTranslation("-3,7,-1", a, -3, 7, -1);
        }

        function testSpecialValues() {
            var a:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
            ]));

            testTranslation("NaN,1,1", a, NaN, 1, 1);
            testTranslation("1,NaN,1,1", a, 1, NaN, 1);
            testTranslation("1,1,NaN,1,1", a, 1, 1, NaN);

            testTranslation("Infinity,1,1", a, Infinity, 1, 1);
            testTranslation("1,Infinity,1,1", a, 1, Infinity, 1);
            testTranslation("1,1,Infinity,1,1", a, 1, 1, Infinity);

            var b:Matrix3D = new Matrix3D(Vector.<Number>([
                0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
            ]));

            testTranslation("NaN,1,1", b, NaN, 1, 1);
            testTranslation("1,NaN,1,1", b, 1, NaN, 1);
            testTranslation("1,1,NaN,1,1", b, 1, 1, NaN);

            testTranslation("Infinity,1,1", b, Infinity, 1, 1);
            testTranslation("1,Infinity,1,1", b, 1, Infinity, 1);
            testTranslation("1,1,Infinity,1,1", b, 1, 1, Infinity);
        }

        function testTranslation(label:String, m:Matrix3D, x:Number, y:Number, z:Number) {
            m = m.clone()
            m.appendTranslation(x,y,z);
            trace(label + " (append): " + m.rawData);

            m = m.clone()
            m.prependTranslation(x,y,z);
            trace(label + " (prepend): " + m.rawData);
        }
    }
}
