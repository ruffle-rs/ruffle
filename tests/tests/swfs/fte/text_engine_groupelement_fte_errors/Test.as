package {
import flash.display.Sprite;
import flash.text.engine.ContentElement;
import flash.text.engine.GroupElement;
import flash.text.engine.TextElement;

public class Test extends Sprite {
    public function Test() {
        var group:GroupElement = makeGroup();
        tryCall("group range", function():void { group.groupElements(2, 1); });
        tryCall("ungroup range", function():void { group.ungroupElements(9); });
        tryCall("ungroup non-group", function():void { group.ungroupElements(0); });
        tryCall("merge range", function():void { group.mergeTextElements(0, 9); });

        var nested:GroupElement = group.groupElements(0, 2);
        tryCall("merge non-text", function():void { group.mergeTextElements(0, 2); });
        trace("nested still grouped: " + nested.text);
    }

    private function makeGroup():GroupElement {
        var vector:Vector.<ContentElement> = new Vector.<ContentElement>();
        vector.push(new TextElement("aa"));
        vector.push(new TextElement("bb"));
        vector.push(new TextElement("cc"));
        return new GroupElement(vector);
    }

    private function tryCall(label:String, f:Function):void {
        try {
            f();
            trace(label + ": no error");
        } catch (e:Error) {
            trace(label + ": " + e.errorID);
        }
    }
}
}
