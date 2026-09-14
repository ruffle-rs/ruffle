package {
    import flash.display.*;
    import flash.text.*;
    import flash.geom.*;

    [SWF(width="500", height="500")]
    public class Test extends Sprite {
        public function Test() {
            super();

            stage.scaleMode = StageScaleMode.NO_SCALE;

            testSetZ();
            trace("");

            testImageComparison();
            trace("");

        }

        private function testSetZ() : void {
            var sprite: Sprite = new Sprite();

            trace("// SetZ: default parameters");
            trace("sprite.z", sprite.z);
            trace("sprite.transform.matrix", sprite.transform.matrix);
            trace("sprite.transform.matrix3D", sprite.transform.matrix3D);

            trace("// SetZ: set .z=0");
            sprite.z = 0;
            trace("sprite.z", sprite.z);
            trace("sprite.transform.matrix", sprite.transform.matrix);
            trace("sprite.transform.matrix3D", sprite.transform.matrix3D);
            trace("sprite.transform.matrix3D.rawData", sprite.transform.matrix3D.rawData);

            trace("// SetZ: set .z=50");
            sprite.z = 50;
            trace("sprite.z", sprite.z);
            trace("sprite.transform.matrix", sprite.transform.matrix);
            trace("sprite.transform.matrix3D", sprite.transform.matrix3D);
            trace("sprite.transform.matrix3D.rawData", sprite.transform.matrix3D.rawData);

            trace("// SetZ: set .z=-50");
            sprite.z = -50;
            trace("sprite.z", sprite.z);
            trace("sprite.transform.matrix", sprite.transform.matrix);
            trace("sprite.transform.matrix3D", sprite.transform.matrix3D);
            trace("sprite.transform.matrix3D.rawData", sprite.transform.matrix3D.rawData);

            trace("// SetZ: set .z=1000000000");
            sprite.z = 1000000000;
            trace("sprite.z", sprite.z);
            trace("sprite.transform.matrix", sprite.transform.matrix);
            trace("sprite.transform.matrix3D", sprite.transform.matrix3D);
            trace("sprite.transform.matrix3D.rawData", sprite.transform.matrix3D.rawData);

            trace("// SetZ: set .z=-1000000000");
            sprite.z = -1000000000;
            trace("sprite.z", sprite.z);
            trace("sprite.transform.matrix", sprite.transform.matrix);
            trace("sprite.transform.matrix3D", sprite.transform.matrix3D);
            trace("sprite.transform.matrix3D.rawData", sprite.transform.matrix3D.rawData);

            trace("// SetZ: set .transform.matrix3D=null");
            sprite.transform.matrix3D = null;
            trace("sprite.z", sprite.z);
            trace("sprite.transform.matrix", sprite.transform.matrix);
            trace("sprite.transform.matrix3D", sprite.transform.matrix3D);

            trace("// SetZ: set .transform.matrix=null");
            sprite.transform.matrix = null;
            trace("sprite.z", sprite.z);
            trace("sprite.transform.matrix", sprite.transform.matrix);
            trace("sprite.transform.matrix3D", sprite.transform.matrix3D);
            trace("sprite.transform.matrix3D.rawData", sprite.transform.matrix3D.rawData);
        }

        private function testImageComparison() : void {
            var s : Sprite = new Sprite();
            stage.addChild(s);

            // right: negative z
            for (var k:int = 0; k < 2; k++) {
                var bd1 : BitmapData = new BitmapData(100, 100, false, 0x0000FFFF - 0x008800 * k);
                var b1 : Bitmap = new Bitmap(bd1);
                b1.x = 498 / 2;
                b1.y = 480 / 2;
                // -490 is not rendered; -450 is rendered.
                b1.z = -490 + (40 * k);
                // FIXME: Ruffle should respect the FP threshold.
                //   In FP, the threshold is at somewhere between -456.233 and -456.234.
                //   In Ruffle, it's at somewhere between -480.245 and -480.246, around the true focalLength.
                // b5.z = -457 + (1 * k);
                s.addChild(b1);
            }

            for (var i:int = 0; i < 8; i++) {
                // top-left
                var bd2 : BitmapData = new BitmapData(100, 100, false, 0xFF00FF - 0x220000 * i);
                var b2 : Bitmap = new Bitmap(bd2);
                b2.z = 50 * i;
                s.addChild(b2);

                // center
                var bd3 : BitmapData = new BitmapData(100, 100, false, 0xFFFF00 - 0x220000 * i);
                var b3 : Bitmap = new Bitmap(bd3);
                b3.x = (500 - bd3.width) / 2;
                b3.y = (500 - bd3.height) / 2;
                b3.z = 50 * i;
                s.addChild(b3);

                // bottom
                var bd4 : BitmapData = new BitmapData(100, 100, false, 0x00FFFF - 0x000022 * i);
                var b4 : Bitmap = new Bitmap(bd4);
                b4.x = (500 - bd4.width) / 2;
                b4.y = (500 - bd4.height);
                b4.z = 50 * i;
                s.addChild(b4);
            }

            // bottom-right: large z to the vanishing point
            for (var j:int = 0; j < 100; j++) {
                var bd5 : BitmapData = new BitmapData(100, 100, false, 0x000000);
                var b5 : Bitmap = new Bitmap(bd5);
                b5.x = 500 - bd5.width;
                b5.y = 500 - bd5.height;
                b5.z = 500 * j;
                s.addChild(b5);
            }
        }
    }
}
