package {
import flash.display.*;
import flash.geom.*;
import flash.utils.*;

[SWF(width="200", height="100")]
public class Test extends MovieClip {
    public function Test() {
        var s:Sprite = new Sprite();
        s.x = 20;
        s.y = 20;
        s.graphics.beginFill(0xFF0000);
        s.graphics.drawRect(0, 0, 60, 60);
        s.graphics.endFill();
        s.blendMode = BlendMode.DIFFERENCE;
        s.transform.colorTransform = new ColorTransform(0,0,0,1,0,255,0,0);

        var mask:Sprite = new Sprite();
        mask.x = -10;
        mask.y = -10;
        mask.graphics.beginFill(0x00FF00);
        mask.graphics.drawRect(0, 0, 35, 30);
        mask.graphics.drawRect(50, 50, 30, 35);
        mask.graphics.endFill();

        s.mask = mask;

        var bd:BitmapData = new BitmapData(100, 100);
        bd.fillRect(new Rectangle(0,0,100,100), 0xFF000000);
        bd.draw(s);
        var b:Bitmap = new Bitmap(bd);
        addChild(b);

        var container:Sprite = new Sprite();
        container.addChild(mask);
        container.addChild(s);
        container.x = 100;
        addChild(container);
    }
}
}
