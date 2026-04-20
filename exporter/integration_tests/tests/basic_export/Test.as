package {
import flash.display.*;

[SWF(width="100", height="100")]
public class Test extends MovieClip {
    public function Test() {
        var s = new Sprite();
        s.graphics.beginFill(0xff0000);
        s.graphics.drawRect(10, 20, 50, 30);
        s.graphics.endFill();
        addChild(s);
    }
}
}
