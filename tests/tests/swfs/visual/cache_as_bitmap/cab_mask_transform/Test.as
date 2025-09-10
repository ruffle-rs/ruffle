package {
import flash.display.*;
import flash.geom.*;

[SWF(width="40", height="40")]
public class Test extends MovieClip {
    [Embed(source="mask.png")]
    private var BitmapMask:Class;
    [Embed(source="maskee.png")]
    private var Maskee:Class;

    public function Test() {
        testCabMask(false, false, x,      y);
        testCabMask(true,  false, x + 20, y);
        testCabMask(false, true,  x,      y + 20);
        testCabMask(true,  true,  x + 20, y + 20);
    }

    // 20x20
    private function testCabMask(maskeeCab:Boolean, maskCab:Boolean, x:int, y:int):void {
        var maskee:Bitmap = new Maskee();
        maskee.cacheAsBitmap = maskeeCab;

        var mask:DisplayObject = new BitmapMask();
        mask.cacheAsBitmap = maskCab;
        mask.x = x + 5;
        mask.y = y + 5;
        mask.transform.colorTransform = new ColorTransform(0,0,0,0.25,0,0,0,64);

        maskee.mask = mask;
        maskee.x = x;
        maskee.y = y;
        addChild(mask);
        addChild(maskee);
    }
}
}
