package {
import flash.display.*;
import flash.geom.*;

[SWF(width="80", height="40")]
public class Test extends MovieClip {
    [Embed(source="maskee.png")]
    private var Maskee:Class;

    public function Test() {
        // graphics non-square semi-transparent
        testMask(function():DisplayObject {
            var s:Sprite = new Sprite();
            s.graphics.beginFill(0xFF0000, 0.5);
            s.graphics.drawTriangles(Vector.<Number>([
                0, 0,
                0, 10,
                10, 10
            ]), Vector.<int>([
                0, 1, 2
            ]));
            s.graphics.endFill();
            return s;
        }, 0, 0);

        // graphics non-square transparent
        testMask(function():DisplayObject {
            var s:Sprite = new Sprite();
            s.graphics.beginFill(0xFF0000, 0);
            s.graphics.drawTriangles(Vector.<Number>([
                0, 0,
                0, 10,
                10, 10
            ]), Vector.<int>([
                0, 1, 2
            ]));
            s.graphics.endFill();
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
