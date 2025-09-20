package {
import flash.display.*;
import flash.geom.*;

[SWF(width="80", height="80")]
public class Test extends MovieClip {
    public function Test() {
        var s:Sprite = new Sprite();
        s.graphics.beginFill(0xFF0000);
        s.graphics.drawRect(10, 10, 20, 20);
        s.graphics.endFill();
        stage.addChild(s);

        s.buttonMode = true;
        stage.focus = s;

        var bdStage:BitmapData = new BitmapData(40, 40);
        bdStage.draw(stage);
        var bitmapStage = new Bitmap(bdStage);
        bitmapStage.x = 40;
        stage.addChild(bitmapStage);

        var bdS:BitmapData = new BitmapData(40, 40);
        bdS.draw(s);
        var bitmapS = new Bitmap(bdS);
        bitmapS.y = 40;
        stage.addChild(bitmapS);

        stage.addEventListener("enterFrame", function(evt:*):void {
            bdStage.draw(stage);
        });
    }
}
}
