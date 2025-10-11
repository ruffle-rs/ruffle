package {
import flash.display.*;
import flash.geom.*;
import flash.filters.*;
import flash.utils.*;

[SWF(width="40", height="40")]
public class Test extends MovieClip {
    public function Test() {
        var s:Sprite = new Sprite();
        s.graphics.beginFill(0xFF00FF);
        s.graphics.drawTriangles(Vector.<Number>([
            0, 0,
            0, 20,
            20, 20
        ]), Vector.<int>([
            0, 1, 2
        ]));
        s.graphics.endFill();
        s.cacheAsBitmap = true;

        var bd:BitmapData = new BitmapData(40, 20);
        bd.fillRect(new Rectangle(0,0,40,20), 0xFF000000);
        bd.drawWithQuality(
            s,
            new Matrix(1, 0, 0, 1, 20, 0),
            new ColorTransform(0, 1, 1),
            "difference",
            new Rectangle(0, 10, 40, 10),
            false,
            "low"
        );
        var b:Bitmap = new Bitmap(bd);
        b.y = 20;
        addChild(b);
        addChild(s);
    }
}
}
