package {
    import flash.display.MovieClip;
    import flash.geom.Matrix;

    public class Test extends MovieClip {
        public function Test() {
            Test_Random();

            Test_NegativeScaleX();
        }

        public function Test_Random() {
            trace("// Test_Random");

            var mc = new MovieClip();
            for(var i:int = -10; i <= 10; i+=5) {
                for(var j:int = -10; j <= 10; j+=5) {
                    for(var k:int = -10; k <= 10; k+=5) {
                        for(var l:int = -10; l <= 10; l+=5) {
                            mc.transform.matrix = new Matrix(i, j, k, l, 0, 0);
                            printTransform(mc);
                        }
                    }
                }
            }
        }

        public function Test_NegativeScaleX() {
            trace("// Test_NegativeScaleX");

            var vals: Array = [-1.1, -1.0, -0.9, -0, 0, 0.9, 1.0, 1.1, 2.0, 3.0, 5.0, 10.0];

            for each(var i in vals) {
                trace(i);
                var mc = new MovieClip();
                mc.transform.matrix = new Matrix(-i, 0, 0, i);
                printTransform(mc);
            }
        }

        private function printTransform(clip:MovieClip) {
            var props = ["rotation", "scaleX", "scaleY"];
            trace("matrix = " + clip.transform.matrix);
            for each (var p in props) {
                trace("  ", p, "=", clip[p]);
            }
        }
    }
}
