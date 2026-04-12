package {

import flash.display.MovieClip;
import flash.display.Sprite;
import flash.display.BlendMode;
import flash.events.Event;

[SWF(width="20", height="20", backgroundColor="#000000")]
public class Test extends MovieClip {
    private var colorB:Sprite;
    private var frame:int = 0;

    public function Test() {
        super();

        var colorA:Sprite = new Sprite();
        colorA.graphics.beginFill(0x0077FF);
        colorA.graphics.drawRect(0, 0, 20, 20);
        colorA.graphics.endFill();

        colorB = new Sprite();
        colorB.graphics.beginFill(0xFF9900);
        colorB.graphics.drawRect(0, 0, 20, 20);
        colorB.graphics.endFill();
        colorB.blendMode = BlendMode.SHADER;

        addChild(colorA);
        addChild(colorB);

        this.addEventListener(Event.ENTER_FRAME, onEnterFrame);
    }

    private function onEnterFrame(e:Event):void {
        this.frame += 1;
        if (this.frame == 2) {
            trace("colorB.blendMode: " + colorB.blendMode);
        }
    }
}
}
