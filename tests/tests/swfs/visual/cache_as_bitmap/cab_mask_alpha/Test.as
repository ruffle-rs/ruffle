package {
import flash.display.*;
import flash.geom.*;

[SWF(width="80", height="40")]
public class Test extends MovieClip {
    [Embed(source="mask.png")]
    private var BitmapMask:Class;
    [Embed(source="maskee.png")]
    private var Maskee:Class;

    public function Test() {
        // bitmap mask
        testMask(function():DisplayObject { return new BitmapMask(); }, 0, 0);

        // graphics mask
        testMask(function():DisplayObject {
            var s:Sprite = new Sprite();
            var colors:Array = [
                0xFF0000,
                0x00FF00,
                0x0000FF,
                0x00FFFF,
                0xFF00FF,
                0xFFFF00,
                0xFFFFFF,
                0x000000,
                0x888888,
                0x444444,
            ];
            var alphas:Array = [
                1.0, 0.8, 0.7, 0.6, 0.5,
                0.4, 0.3, 0.2, 0.1, 0.0
            ];
            var i:uint = 0;
            while (i < 10) {
                var j:uint = 0;
                while (j < 10) {
                    s.graphics.beginFill(colors[j], alphas[i]);
                    s.graphics.drawRect(i,j,1,1);
                    s.graphics.endFill();
                    ++j;
                }
                ++i;
            }
            return s;
        }, 40, 0);
    }

    // 40x40
    private function testMask(maskFactory:Function, x:int, y:int):void {
        testCabMask(maskFactory, false, false, x,      y);
        testCabMask(maskFactory, true,  false, x + 20, y);
        testCabMask(maskFactory, false, true,  x,      y + 20);
        testCabMask(maskFactory, true,  true,  x + 20, y + 20);
    }

    // 20x20
    private function testCabMask(maskFactory:Function, maskeeCab:Boolean, maskCab:Boolean, x:int, y:int):void {
        var maskee:Bitmap = new Maskee();
        maskee.cacheAsBitmap = maskeeCab;

        var mask:DisplayObject = maskFactory();
        mask.cacheAsBitmap = maskCab;
        mask.x = x + 5;
        mask.y = y + 5;

        maskee.mask = mask;
        maskee.x = x;
        maskee.y = y;
        addChild(mask);
        addChild(maskee);
    }
}
}
