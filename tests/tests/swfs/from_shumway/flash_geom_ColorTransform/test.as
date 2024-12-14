/*
   Compiled with:
   node utils/compileabc.js --swf ColorTransformTest,100,100,60 -p test/swfs/flash_geom_ColorTransform.as
*/

﻿package  {
    import flash.display.Sprite;
    import flash.display.GradientType;
    import flash.geom.ColorTransform;
    import flash.events.MouseEvent;

    public class ColorTransformTest extends Sprite {
        public function ColorTransformTest() {
					createSprite(0, 0, new ColorTransform(0.5, 0.2, 1, 0.4, 0, 200, 0, 0));
					createSprite(20, 0, new ColorTransform(0.5, 0.2, 1, 0, 0, 200, 0, 255));
					createSprite(40, 0, new ColorTransform(0.5, 0.2, 1, 1, 0, 200, 0, 0));

					createSprite(0, 20, new ColorTransform(1, 0, 0, 1, 0, 0, 0, 0));
					createSprite(20, 20, new ColorTransform(0, 1, 0, 1, 0, 0, 0, 0));
					createSprite(40, 20, new ColorTransform(0, 0, 1, 1, 0, 0, 0, 0));
					createSprite(60, 20, new ColorTransform(0, 0, 0, 1, 0, 0, 0, 0));
					createSprite(80, 20, new ColorTransform(1, 1, 1, 1, 0, 0, 0, 0));

					createSprite(0, 40, new ColorTransform(0, 0, 0, 1, 255, 0, 0, 0));
					createSprite(20, 40, new ColorTransform(0, 0, 0, 1, 0, 255, 0, 0));
					createSprite(40, 40, new ColorTransform(0, 0, 0, 1, 0, 0, 255, 0));
					createSprite(60, 40, new ColorTransform(0, 0, 0, 1, 0, 0, 0, -255));
					createSprite(80, 40, new ColorTransform(1, 1, 1, 1, 0, 0, 0, 0));

					createSprite(0, 60, new ColorTransform(-1, 0, 0, 1, 255, 255, 255, 0));
					createSprite(20, 60, new ColorTransform(0, -1, 0, 1, 255, 255, 255, 0));
					createSprite(40, 60, new ColorTransform(0, 0, -1, 1, 255, 255, 255, 0));
					createSprite(60, 60, new ColorTransform(0, 0, 0, -1, 0, 0, 0, 0));
					createSprite(80, 60, new ColorTransform(-1, -1, -1, 1, 255, 255, 255, 0));

					createSprite(0, 80, new ColorTransform(1, 1, 1, 0.4, 0, 0, 0, 0), 0x00FF00);
					createSprite(20, 80, new ColorTransform(1, 1, 1, 0, 0, 0, 0, 255), 0x00FF00);
					createSprite(40, 80, new ColorTransform(1, 1, 1, 0, 0, 0, 0, 0), 0xFF0000);
        }
				private function createSprite(x:int, y:int, ct:ColorTransform, c:uint = 0xFFFFFF) {
            var sprite:Sprite = new Sprite();
            sprite.graphics.beginFill(c);
            sprite.graphics.drawRect(0, 0, 20, 20);
						sprite.x = x;
						sprite.y = y;
            sprite.transform.colorTransform = ct;
            addChild(sprite);
        }
    }
}
