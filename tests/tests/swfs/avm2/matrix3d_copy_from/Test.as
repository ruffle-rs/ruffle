package {
    import flash.display.Sprite;
    import flash.geom.Matrix3D;

    public class Test extends Sprite {
        public function Test() {
            testBasic();
            testIndependence();
            testSelfCopy();
            testNull();
        }

        function testBasic() {
            trace("// testBasic");
            var source:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
                13, 14, 15, 16
            ]));
            var dest:Matrix3D = new Matrix3D();
            dest.copyFrom(source);
            trace(source.rawData);
            trace(dest.rawData);
        }

        function testIndependence() {
            trace("// testIndependence");
            var source:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
                13, 14, 15, 16
            ]));
            var dest:Matrix3D = new Matrix3D();
            dest.copyFrom(source);

            source.appendTranslation(100, 200, 300);
            trace("source:");
            trace(source.rawData);
            trace("dest:");
            trace(dest.rawData);

            dest.appendTranslation(-1, -2, -3);
            trace("source:");
            trace(source.rawData);
            trace("dest:");
            trace(dest.rawData);
        }

        function testSelfCopy() {
            trace("// testSelfCopy");
            var matrix:Matrix3D = new Matrix3D(Vector.<Number>([
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
                13, 14, 15, 16
            ]));
            matrix.copyFrom(matrix);
            trace(matrix.rawData);
        }

        function testNull() {
            trace("// testNull");
            try {
                new Matrix3D().copyFrom(null);
                trace("Didn't throw");
            } catch (e) {
                trace("Caught error: " + e.getStackTrace());
            }
        }
    }
}
