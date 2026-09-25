package {

import flash.display.Sprite;
import flash.text.engine.*;
import flash.events.EventDispatcher;

public class Test extends Sprite {
    public function Test() {
        trace(ContentElement.GRAPHIC_ELEMENT);

        try {
            new ContentElement();
        } catch (e) {
            trace(e.getStackTrace());
        }

        var e:ContentElement = new CustomContentElement();
        trace(e.userData);
        trace(e.textBlock);
        trace(e.textBlockBeginIndex);
        trace(e.elementFormat);
        trace(e.eventMirror);
        trace(e.groupElement);
        trace(e.rawText);
        trace(e.text);
        trace(e.textRotation);

        trace("userData");
        var getter = function() { return e.userData; };
        testSetter(getter, function() { e.userData = null; });
        testSetter(getter, function() { e.userData = "5"; });
        testSetter(getter, function() { e.userData = 12; });
        testSetter(getter, function() { e.userData = false; });
        testSetter(getter, function() { e.userData = undefined; });
        testSetter(getter, function() { e.userData = "hello"; });

        trace("elementFormat");
        var getter = function() { return e.elementFormat; };
        testSetter(getter, function() { e.elementFormat = null; });
        testSetter(getter, function() { e.elementFormat = new ElementFormat(); });

        trace("eventMirror");
        var getter = function() { return e.eventMirror; };
        testSetter(getter, function() { e.eventMirror = null; });
        testSetter(getter, function() { e.eventMirror = new EventDispatcher(); });

        trace("textRotation");
        var getter = function() { return e.textRotation; };
        testSetter(getter, function() { e.textRotation = null; });
        testSetter(getter, function() { e.textRotation = ""; });
        testSetter(getter, function() { e.textRotation = "<invalid>"; });
        testSetter(getter, function() { e.textRotation = "auto"; });
        testSetter(getter, function() { e.textRotation = "AUTO"; });
        testSetter(getter, function() { e.textRotation = "rotate0"; });
        testSetter(getter, function() { e.textRotation = "Rotate0"; });
        testSetter(getter, function() { e.textRotation = "rotate90"; });
        testSetter(getter, function() { e.textRotation = "rotate180"; });
        testSetter(getter, function() { e.textRotation = "rotate270"; });
    }

    private function testSetter(getter:Function, setter:Function) {
        try {
            setter();
        } catch (e) {
            trace("  Caught error:" + e.getStackTrace().split("\n").slice(0, 2).join("\n"));
        }

        trace("  Value:" + getter());
    }
}

}
