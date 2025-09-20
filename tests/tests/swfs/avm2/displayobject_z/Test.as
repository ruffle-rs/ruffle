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

            trace("// SetZ: set .z=50");
            sprite.z = 50;
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

            for (var i:int = 0; i < 8; i++) {
                // top-left
                var bd1 : BitmapData = new BitmapData(100, 100, false, 0xFF00FF - 0x220000 * i);
                var b1 : Bitmap = new Bitmap(bd1);
                b1.z = 50 * i;
                s.addChild(b1);

                // center
                var bd2 : BitmapData = new BitmapData(100, 100, false, 0xFFFF00 - 0x220000 * i);
                var b2 : Bitmap = new Bitmap(bd2);
                b2.x = (stage.stageWidth - bd2.width) / 2;
                b2.y = (stage.stageHeight - bd2.height) / 2;
                b2.z = 50 * i;
                s.addChild(b2);

                // bottom
                var bd3 : BitmapData = new BitmapData(100, 100, false, 0x00FFFF - 0x000022 * i);
                var b3 : Bitmap = new Bitmap(bd3);
                b3.x = (stage.stageWidth - bd3.width) / 2;
                b3.y = (stage.stageHeight - bd3.height);
                b3.z = 50 * i;
                s.addChild(b3);
            }

            for (var j:int = 0; j < 100; j++) {
                var bd4 : BitmapData = new BitmapData(100, 100, false, 0x000000);
                var b4 : Bitmap = new Bitmap(bd4);
                b4.x = stage.stageWidth - bd4.width;
                b4.y = stage.stageHeight - bd4.height;
                b4.z = 500 * j;
                s.addChild(b4);
            }
        }
    }
}
