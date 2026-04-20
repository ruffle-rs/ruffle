package {
import flash.display.*;
import flash.geom.*;

[SWF(width="10", height="10")]
public class Test extends Sprite {
    public function Test() {
        var st = new Error().getStackTrace();
        var rect = new Sprite();
        if (st === null) {
            rect.graphics.beginFill(0xFF0000);
        } else if (st === undefined) {
            rect.graphics.beginFill(0x00FF00);
        } else {
            rect.graphics.beginFill(0x0000FF);
        }
        rect.graphics.drawRect(0, 0, 10, 10);
        rect.graphics.endFill();
        addChild(rect);
    }
}
}
