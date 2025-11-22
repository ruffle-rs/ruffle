package {

import flash.display.*;
import flash.text.*;

[SWF(width="100", height="100", backgroundColor="#FF00FF")]
public class Test extends MovieClip {
    public function Test() {
        var text:TextField = createTextField();
        addChild(text);

        trace("Loaded test!");

        trace("Character bounds:");
        trace(text.getCharBoundaries(0).x + text.x);
        trace(text.getCharBoundaries(0).width);
        trace(text.getCharBoundaries(0).y + text.y);
        trace(text.getCharBoundaries(0).height);
    }

    private function createTextField():TextField {
        var text:TextField = new TextField();
        text.text = "M";
        text.x = 20;
        text.y = 20;
        text.width = 80;
        text.height = 80;
        return text;
    }
}

}
