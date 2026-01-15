package {
import flash.display.*;
import flash.filters.*;
import flash.geom.*;

[SWF(width="80", height="40")]
public class Test extends MovieClip {
    [Embed(source="mask.png")]
    private var BitmapMask:Class;
    [Embed(source="maskee.png")]
    private var Maskee:Class;

    public function Test() {
        // blur
        testFilters([new BlurFilter(5, 5)], 0, 0);

        // drop shadow
        testFilters([
            new DropShadowFilter(1, 180, 0, 0.5, 0, 0),
            new DropShadowFilter(1, 90, 0, 1, 0, 0)
        ], 40, 0);
    }

    // 40x40
    private function testFilters(filters:Array, x:int, y:int):void {
        testCabMask(filters, false, false, x,      y);
        testCabMask(filters, true,  false, x + 20, y);
        testCabMask(filters, false, true,  x,      y + 20);
        testCabMask(filters, true,  true,  x + 20, y + 20);
    }

    // 20x20
    private function testCabMask(filters:Array, maskeeCab:Boolean, maskCab:Boolean, x:int, y:int):void {
        var maskee:Bitmap = new Maskee();
        maskee.cacheAsBitmap = maskeeCab;

        var mask:DisplayObject = new BitmapMask();
        mask.cacheAsBitmap = maskCab;
        mask.x = x + 5;
        mask.y = y + 5;
        mask.filters = filters;

        maskee.mask = mask;
        maskee.x = x;
        maskee.y = y;
        addChild(mask);
        addChild(maskee);
    }
}
}
