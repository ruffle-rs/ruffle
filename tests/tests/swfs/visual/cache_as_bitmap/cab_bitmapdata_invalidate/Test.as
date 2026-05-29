package {

import flash.display.*;

[SWF(width="5", height="5")]
public class Test extends Sprite {
    private var frame:int = 0;

    public function Test() {
        var bd:BitmapData = new BitmapData(5, 5, false, 0x00FF000);
        var b:Bitmap = new Bitmap(bd);
        b.cacheAsBitmap = true;
        addChild(b);

        addEventListener("enterFrame", function(evt:*):void {
            if (frame == 10) {
                var bd:BitmapData = new BitmapData(5, 5, false, 0xFF00FF);
                b.bitmapData = bd;
            }

            frame += 1;
        });
    }
}

}
