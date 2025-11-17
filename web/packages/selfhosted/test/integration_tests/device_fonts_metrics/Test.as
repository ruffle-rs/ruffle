package {

import flash.display.*;
import flash.text.*;

[SWF(width="100", height="100")]
public class Test extends MovieClip {
    public function Test() {
        var text:TextField = createTextField();
        addChild(text);

        trace("Loaded test!");

        trace("Displayed text metrics:");
        trace(text.textWidth);
        trace(text.textHeight);
        trace(text.getCharBoundaries(0).x);
        trace(text.getCharBoundaries(0).width);
        trace(text.getCharBoundaries(1).x);
        trace(text.getCharBoundaries(1).width);
        trace(text.getCharBoundaries(2).x);
        trace(text.getCharBoundaries(2).width);

        if (text.getCharBoundaries(3) == null) {
            trace("null");
            trace("null");
        } else {
            trace(text.getCharBoundaries(3).x);
            trace(text.getCharBoundaries(3).width);
        }
    }

    private function createTextField():TextField {
        var text:TextField = new TextField();
        text.text = "MxT體";
        text.x = 10;
        text.y = 10;
        text.width = 80;
        text.height = 80;
        return text;
    }
}

}
