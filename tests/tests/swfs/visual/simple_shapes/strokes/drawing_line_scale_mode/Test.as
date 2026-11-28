package {

import flash.display.LineScaleMode;
import flash.display.MovieClip;
import flash.display.Shape;

// Stroke width of drawing API shapes should follow the line's scale mode:
// NONE keeps a 1px stroke at any scale, the other modes scale it.
[SWF(width="330", height="190", backgroundColor="#FFFFFF")]
public class Test extends MovieClip {
    public function Test() {
        // A non-scaling stroke on circles scaled 1x, 2x and 3.5x.
        addCircle(30, 45, 1, LineScaleMode.NONE);
        addCircle(80, 45, 2, LineScaleMode.NONE);
        addCircle(160, 45, 3.5, LineScaleMode.NONE);

        // Every scale mode at a uniform 3x scale.
        addCircle(40, 140, 3, LineScaleMode.NORMAL);
        addCircle(120, 140, 3, LineScaleMode.HORIZONTAL);
        addCircle(200, 140, 3, LineScaleMode.VERTICAL);
        addCircle(280, 140, 3, LineScaleMode.NONE);
    }

    private function addCircle(x:Number, y:Number, scale:Number, scaleMode:String):void {
        var shape:Shape = new Shape();
        shape.graphics.lineStyle(1, 0x000000, 1, false, scaleMode);
        shape.graphics.drawCircle(0, 0, 10);
        shape.x = x;
        shape.y = y;
        shape.scaleX = scale;
        shape.scaleY = scale;
        addChild(shape);
    }
}
}
