package {
    import flash.display.MovieClip;
    import flash.geom.Matrix;

    public class Test extends MovieClip {
        public function Test() {
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

        private function printTransform(clip:MovieClip) {
            var props = ["rotation", "scaleX", "scaleY"];
            trace("matrix = " + clip.transform.matrix);
            for each (var p in props) {
                trace("  ", p, "=", clip[p]);
            }
        }
    }
}
