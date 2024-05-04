package {

import flash.display.Sprite;
import flash.text.TextField;
import flash.display.Sprite;
import flash.display.Shape;
import flash.display.MovieClip;
import flash.events.FocusEvent;

[SWF(width="50", height="50", backgroundColor="#000000")]
public class Test extends MovieClip {
    public function Test() {
        super();

        var shape = new Shape();
        shape.graphics.beginFill(0xFFFF0000);
        shape.graphics.drawRect(10, 10, 30, 30);
        shape.graphics.endFill();
        this.stage.addChild(shape);
        this.stage.addEventListener("focusIn", function (evt:FocusEvent):void {
            trace("Focus changed to stage");
        });
        this.stage.focus = this.stage;
    }
}
}
