package {
    import flash.display.Sprite;
    import flash.geom.Matrix;

    // The scaleX, scaleY and rotation setters rebuild the matrix in 16.16 fixed point, so
    // every entry is traced as a multiple of 1/65536.
    public class Test extends Sprite {
        public function Test() {
            var s:Sprite;
            var v:Number;
            var i:int;

            trace("// scaleX");
            var scales:Array = [1, 2, 0.5, 0.3, 0.33, 0.777, 1.7, 2.6, 0.00375, 0.00625, 0.00875, 0.01125,
                -0.05, -0.3, -1.5, 0.000001, -0.00001, 0, 1e-10, 100, 32767, 32767.5, 32768, 32769, 40000,
                65536, 1e10, -32768, -32769, -1e10, Infinity, -Infinity, NaN];
            for each (v in scales) {
                s = new Sprite();
                s.scaleX = v;
                trace(v + " -> " + entries(s) + " scaleX " + s.scaleX);
            }

            trace("// scaleY");
            for each (v in [0.3, 2.6, -0.05, 32768, NaN]) {
                s = new Sprite();
                s.scaleY = v;
                trace(v + " -> " + entries(s) + " scaleY " + s.scaleY);
            }

            trace("// rotation, every 7.25 degrees");
            for (i = -52; i <= 52; i++) {
                rotated(i * 7.25);
            }

            trace("// rotation, between multiples of 0.25");
            var starts:Array = [0, 44.5, 89.5, 137, 179.5, 269.5, -0.5, -90.5];
            for each (var start:Number in starts) {
                for (i = 0; i <= 50; i++) {
                    rotated(start + i / 50);
                }
            }
            for (i = 0; i <= 25; i++) {
                rotated(10 + i / 100);
            }

            trace("// rotation, out of the ordinary");
            for each (v in [360, 720.3, -720.3, 1000000.1, 32767.3, 32768.3, 65536.3, 1e10, -1e10,
                    Infinity, -Infinity, NaN, 0.00001, -0.00001]) {
                rotated(v);
            }

            trace("// rotation too large for 16.16, read back and then scaled");
            for each (v in [32768.3, 1000000.1, -1e10]) {
                s = new Sprite();
                s.rotation = v;
                trace(v + " -> rotation " + s.rotation);
                s.scaleX = 2;
                trace(v + ", scaleX 2 -> " + entries(s) + " rotation " + s.rotation);
                s.rotation = s.rotation;
                trace(v + ", rotation as read -> " + entries(s));
            }

            trace("// scale, then rotation");
            var pairs:Array = [[2.5, 2], [0.3, 0.3], [-1.5, 0.75], [0.33, 7.77], [32767, 1], [Infinity, 1], [NaN, 2]];
            var angles:Array = [0.01, 0.1, 10, 30, 90, 137.3, -20];
            for each (var pair:Array in pairs) {
                for each (v in angles) {
                    s = new Sprite();
                    s.scaleX = pair[0];
                    s.scaleY = pair[1];
                    s.rotation = v;
                    trace(pair[0] + " x " + pair[1] + ", " + v + " -> " + entries(s));
                }
            }

            trace("// products that land exactly halfway");
            var ties:Array = [[-0.75, 1, 137.3], [0.75, 1, -137.3], [1, 0.75, 137.3], [1, 0.75, -137.3],
                [1, -0.75, 137.3], [-0.75, -0.75, 42.7], [0.75, 0.75, 47.3], [0.75, -0.75, 132.7]];
            for each (var tie:Array in ties) {
                s = new Sprite();
                s.scaleX = tie[0];
                s.scaleY = tie[1];
                s.rotation = tie[2];
                trace(tie + " -> " + entries(s));
            }

            trace("// rotation, then scale");
            for each (pair in pairs) {
                s = new Sprite();
                s.rotation = 30;
                s.scaleX = pair[0];
                trace("30, scaleX " + pair[0] + " -> " + entries(s));
                s.scaleY = pair[1];
                trace("30, scaleY " + pair[1] + " -> " + entries(s));
            }

            trace("// NaN rotation after a real one");
            s = new Sprite();
            s.rotation = 30;
            s.rotation = NaN;
            trace(entries(s) + " rotation " + s.rotation);
            s = new Sprite();
            s.scaleX = 2;
            s.scaleY = 3;
            s.rotation = 30;
            s.rotation = NaN;
            trace(entries(s) + " rotation " + s.rotation);

            trace("// a setter rewrites the whole matrix");
            s = assigned(0.3, 0, 0, 0.3);
            s.scaleX = 0.7;
            trace("scaleX -> " + entries(s));
            s = assigned(0.3, 0, 0, 0.3);
            s.scaleY = 0.5;
            trace("scaleY -> " + entries(s));
            s = assigned(0.3, 0, 0, 0.3);
            s.rotation = 10;
            trace("rotation -> " + entries(s));

            trace("// unless it is given the value it already has");
            s = assigned(0.3, 0, 0, 0.3);
            s.scaleX = s.scaleX;
            s.scaleY = s.scaleY;
            s.rotation = s.rotation;
            trace("unscaled -> " + entries(s));
            s = assigned(0.3, 0.1, -0.1, 0.3);
            s.scaleX = s.scaleX;
            s.scaleY = s.scaleY;
            s.rotation = s.rotation;
            trace("rotated -> " + entries(s));
            s = assigned(0.3, 0, 0, 0.3);
            s.rotation = 360;
            trace("a full turn is not the same value -> " + entries(s));

            trace("// skewed matrices");
            for each (var m:Array in [[1, 0.5, 0.25, 1], [2, 0, 1, 1]]) {
                s = assigned(m[0], m[1], m[2], m[3]);
                s.rotation = 10;
                trace(m + ", rotation 10 -> " + entries(s));
                s.rotation = 33.3;
                trace(m + ", rotation 33.3 -> " + entries(s));
                s.scaleX = 2;
                trace(m + ", scaleX 2 -> " + entries(s));
                s.scaleY = 0.3;
                trace(m + ", scaleY 0.3 -> " + entries(s));
            }

            trace("// width and height");
            s = new Sprite();
            s.graphics.beginFill(0);
            s.graphics.drawRect(0, 0, 100, 100);
            s.width = 33;
            s.height = 77.7;
            trace(entries(s) + " scaleX " + s.scaleX + " scaleY " + s.scaleY);
        }

        private function rotated(rotation:Number):void {
            var s:Sprite = new Sprite();
            s.rotation = rotation;
            trace(rotation + " -> " + entries(s));
        }

        private function assigned(a:Number, b:Number, c:Number, d:Number):Sprite {
            var s:Sprite = new Sprite();
            s.transform.matrix = new Matrix(a, b, c, d);
            return s;
        }

        private function entries(s:Sprite):String {
            var m:Matrix = s.transform.matrix;
            return (m.a * 65536) + " " + (m.b * 65536) + " " + (m.c * 65536) + " " + (m.d * 65536);
        }
    }
}
