package {
    import flash.display.Sprite;
    import flash.geom.Rectangle;

    [SWF(width="600", height="440", backgroundColor="#FFFFFF")]
    public class Test extends Sprite {
        public function Test() {
            testRotationWrap();
            testEulerMatrix3D();
            testHitTestProjected();
        }

        private function box(w:Number, h:Number, color:uint):Sprite {
            var s:Sprite = new Sprite();
            s.graphics.beginFill(color);
            s.graphics.drawRect(0, 0, w, h);
            s.graphics.endFill();
            return s;
        }

        private function round3(n:Number):Number {
            return Math.round(n * 1000) / 1000;
        }

        private function dumpRawData(label:String, sprite:Sprite):void {
            var data:Vector.<Number> = sprite.transform.matrix3D.rawData;
            var parts:Array = [];
            for (var i:int = 0; i < 16; i++) {
                parts.push(round3(data[i]));
            }
            trace(label, parts.join(","));
        }

        private function testRotationWrap():void {
            trace("// 2D rotation wraps into [-180, 180]");
            var twoD:Sprite = new Sprite();
            twoD.rotation = 270;
            trace("rotation=270 ->", twoD.rotation);
            twoD.rotation = -270;
            trace("rotation=-270 ->", twoD.rotation);
            twoD.rotation = 180;
            trace("rotation=180 ->", twoD.rotation);

            trace("// 3D rotation keeps the native heading (no wrap at ±180)");
            var threeD:Sprite = new Sprite();
            threeD.rotationX = 0;
            threeD.rotation = 270;
            trace("rotationX=0, rotation=270 ->", threeD.rotation);
            threeD.rotation = 270 + 90;
            trace("rotation += 90 ->", threeD.rotation);
            threeD.rotationZ = -90;
            trace("rotationZ=-90 ->", threeD.rotation, "rotationZ", threeD.rotationZ);
        }

        private function testEulerMatrix3D():void {
            trace("// matrix3D is T * Rz * Ry * Rx * S");
            var s:Sprite = new Sprite();
            s.x = 10;
            s.y = 20;
            s.z = 5;
            dumpRawData("z only", s);

            s = new Sprite();
            s.rotationX = 90;
            dumpRawData("rotationX=90", s);

            s = new Sprite();
            s.rotation = 90;
            s.rotationX = 90;
            dumpRawData("rotation=90, rotationX=90", s);
        }

        private function testHitTestProjected():void {
            // Mimic a pitched camera: ENGINE sits at y=500 on a 600x440
            // stage, then parent rotationX brings that 2D-offstage clip
            // into view. 2D getBounds stay at y=500; hitTestObject should
            // still see the projected silhouette.
            var world:Sprite = new Sprite();
            addChild(world);
            world.rotationX = -60;

            var stageSquare:Sprite = box(600, 440, 0x0000FF);
            addChild(stageSquare);

            var onStage:Sprite = box(40, 40, 0x00FF00);
            onStage.x = 280;
            onStage.y = 120;
            world.addChild(onStage);

            var ahead:Sprite = box(40, 40, 0xFF0000);
            ahead.x = 280;
            ahead.y = 500;
            world.addChild(ahead);

            trace("// hitTestObject against a stage rectangle");
            trace("onStage vs stageSquare", onStage.hitTestObject(stageSquare));
            trace("ahead vs stageSquare", ahead.hitTestObject(stageSquare));

            var onStageBounds:Rectangle = onStage.getBounds(this);
            var aheadBounds:Rectangle = ahead.getBounds(this);
            trace(
                "onStage getBounds",
                round3(onStageBounds.x),
                round3(onStageBounds.y),
                round3(onStageBounds.width),
                round3(onStageBounds.height)
            );
            trace(
                "ahead getBounds",
                round3(aheadBounds.x),
                round3(aheadBounds.y),
                round3(aheadBounds.width),
                round3(aheadBounds.height)
            );
        }
    }
}
