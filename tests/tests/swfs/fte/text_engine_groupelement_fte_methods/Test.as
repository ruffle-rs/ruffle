package {
import flash.display.Sprite;
import flash.text.engine.ContentElement;
import flash.text.engine.GroupElement;
import flash.text.engine.TextElement;

public class Test extends Sprite {
    public function Test() {
        var group:GroupElement = makeGroup();
        trace("char 0: " + group.getElementAtCharIndex(0).text);
        trace("char 1: " + group.getElementAtCharIndex(1).text);
        trace("char 2: " + group.getElementAtCharIndex(2).text);
        trace("char 5: " + group.getElementAtCharIndex(5).text);
        trace("char 6: " + group.getElementAtCharIndex(6));

        var nested:GroupElement = group.groupElements(1, 3);
        trace("after group count: " + group.elementCount);
        trace("nested count: " + nested.elementCount);
        trace("nested text: " + nested.text);
        trace("outer text: " + group.text);

        group.ungroupElements(1);
        trace("after ungroup count: " + group.elementCount);
        trace("after ungroup text: " + group.text);

        var merged:TextElement = group.mergeTextElements(0, 2);
        trace("merged text: " + merged.text);
        trace("after merge count: " + group.elementCount);
        trace("after merge text: " + group.text);
    }

    private function makeGroup():GroupElement {
        var vector:Vector.<ContentElement> = new Vector.<ContentElement>();
        vector.push(new TextElement("aa"));
        vector.push(new TextElement("bbb"));
        vector.push(new TextElement("c"));
        return new GroupElement(vector);
    }
}
}
