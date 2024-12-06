package {
    import flash.display.*;
    import flash.text.*;
    import flash.geom.*;

    [SWF(width="500", height="500")]
    public class Test extends MovieClip {
        public function Test() {
            super();

            test2D();
            trace("");

            test3D();
            trace("");

            testImageComparison();
        }

        private function test2D() : void {
            var sprite2D : Sprite = new Sprite();
            var mat2D : Matrix = new Matrix();
            mat2D.identity();

            trace("// sprite2D: new Sprite has null matrix3D and valid matrix");
            trace("sprite2D.transform.matrix", sprite2D.transform.matrix);
            trace("sprite2D.transform.matrix3D", sprite2D.transform.matrix3D);

            trace("// sprite2D: set identity matrix");
            sprite2D.transform.matrix = mat2D;
            trace("sprite2D.transform.matrix", sprite2D.transform.matrix);
            trace("sprite2D.transform.matrix3D", sprite2D.transform.matrix3D);
            trace("mat2D", mat2D);

            trace("// sprite2D: set x = 30, y = 50");
            sprite2D.x = 30;
            sprite2D.y = 50;
            trace("sprite2D.transform.matrix", sprite2D.transform.matrix);
            trace("sprite2D.transform.matrix3D", sprite2D.transform.matrix3D);
            trace("mat2D", mat2D);
        }

        private function test3D() : void {
            var sprite3D : Sprite = new Sprite();
            var mat3D : Matrix3D = new Matrix3D();
            mat3D.identity();

            trace("// sprite3D: set identity matrix3D");
            sprite3D.transform.matrix3D = mat3D;
            trace("sprite3D.transform.matrix", sprite3D.transform.matrix);
            trace("sprite3D.transform.matrix3D", sprite3D.transform.matrix3D);
            trace("sprite3D.transform.matrix3D.rawData", sprite3D.transform.matrix3D.rawData);
            trace("mat3D.rawData", mat3D.rawData);

            trace("// sprite3D: set x = 30, y = 50");
            sprite3D.x = 30;
            sprite3D.y = 50;
            trace("sprite3D.transform.matrix", sprite3D.transform.matrix);
            trace("sprite3D.transform.matrix3D", sprite3D.transform.matrix3D);
            trace("sprite3D.transform.matrix3D.rawData", sprite3D.transform.matrix3D.rawData);
            trace("mat3D.rawData", mat3D.rawData);
        }

        private function testImageComparison() : void {
            var m : Matrix3D = new Matrix3D();

            // id
            var s1 : Sprite = new Sprite();
            s1.x = 10;
            s1.y = 10;
            var bd1 : BitmapData = new BitmapData(50, 50, false, 0xFF0000);
            var b1 : Bitmap = new Bitmap(bd1);
            m.identity();
            b1.transform.matrix3D = m.clone();
            s1.addChild(b1);
            addChild(s1);

            // scale
            var s2 : Sprite = new Sprite();
            s2.x = 160;
            s2.y = 10;
            var bd2 : BitmapData = new BitmapData(50, 50, false, 0x00FF00);
            var b2 : Bitmap = new Bitmap(bd2);
            m.identity();
            m.appendScale(1.5, 3, 1);
            b2.transform.matrix3D = m.clone();
            s2.addChild(b2);
            addChild(s2);

            // rotation
            var s3 : Sprite = new Sprite();
            s3.x = 310;
            s3.y = 10;
            var bd3 : BitmapData = new BitmapData(50, 50, false, 0x00FFFF);
            var b3 : Bitmap = new Bitmap(bd3);
            m.identity();
            m.appendRotation(30, Vector3D.Z_AXIS);
            b3.transform.matrix3D = m.clone();
            s3.addChild(b3);
            addChild(s3);

            // translation
            var s4 : Sprite = new Sprite();
            s4.x = 10;
            s4.y = 160;
            var bd4 : BitmapData = new BitmapData(50, 50, false, 0x0000FF);
            var b4 : Bitmap = new Bitmap(bd4);
            m.identity();
            m.appendTranslation(50, 50, 0);
            b4.transform.matrix3D = m.clone();
            s4.addChild(b4);
            addChild(s4);

            // scale + rotation + translation
            var s5 : Sprite = new Sprite();
            s5.x = 160;
            s5.y = 160;
            var bd5 : BitmapData = new BitmapData(50, 50, false, 0xFF00FF);
            var b5 : Bitmap = new Bitmap(bd5);
            m.identity();
            m.appendScale(2, 3, 1);
            m.appendRotation(30, Vector3D.Z_AXIS);
            m.appendTranslation(50, 50, 0);
            b5.transform.matrix3D = m.clone();
            s5.addChild(b5);
            addChild(s5);
        }
    }
}
