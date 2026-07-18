package {

import flash.display.Sprite;
import flash.text.engine.*;

public class Test extends Sprite {
    public function Test() {
        var e:TextElement = new TextElement();
        trace(e.userData);
        trace(e.textBlock);
        trace(e.textBlockBeginIndex);
        trace(e.elementFormat);
        trace(e.eventMirror);
        trace(e.groupElement);
        trace(e.rawText);
        trace(e.text);
        trace(e.textRotation);

        trace("text");
        var getter = function() { return e.text; };
        testSetter(getter, function() { e.text = null; });
        testSetter(getter, function() { e.text = "null"; });
        testSetter(getter, function() { e.text = ""; });
        testSetter(getter, function() { e.text = "hello"; });

        testReplace(e, 0, 0, "x");
        testReplace(e, 0, 2, "y");
        testReplace(e, -1, 1, "z");
        testReplace(e, 15, 16, "u");
        testReplace(e, 2, 1, "i");
        e.text = null;
        testReplace(e, 0, 1, "p");
        testReplace(e, 0, 0, "p");
    }

    private function testSetter(getter:Function, setter:Function) {
        try {
            setter();
        } catch (e) {
            trace("  Caught error:" + e.getStackTrace().split("\n").slice(0, 2).join("\n"));
        }

        var val = getter();
        if (val == null) {
            trace("  Value:(null)");
        } else {
            trace("  Value:" + val);
        }
    }

    private function testReplace(e:TextElement, from:int, to:int, s:String) {
        trace("Replacing " + from + ", " + to + ", " + s + ":");
        try {
            e.replaceText(from, to, s);
        } catch (e) {
            trace("  Caught error:" + e.getStackTrace().split("\n").slice(0, 2).join("\n"));
        }

        var val = e.text;
        if (val == null) {
            trace("  Value:(null)");
        } else {
            trace("  Value:" + val);
        }
    }
}

}
