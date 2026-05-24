package {
import flash.display.Sprite;
import flash.text.engine.ContentElement;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.GroupElement;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";

        var small:ElementFormat = new ElementFormat(fd, 12);
        var large:ElementFormat = new ElementFormat(fd, 36);

        var plain:TextLine = new TextBlock(new TextElement("WW", small)).createTextLine(null, 400);
        var parts:Vector.<ContentElement> = new Vector.<ContentElement>();
        parts.push(new TextElement("W", small));
        parts.push(new TextElement("W", large));
        var grouped:TextLine = new TextBlock(new GroupElement(parts)).createTextLine(null, 400);

        trace("group raw length: " + grouped.rawTextLength);
        trace("group atom count: " + grouped.atomCount);
        trace("second child affects width: " + (grouped.textWidth > plain.textWidth * 2));
        trace("second atom wider: " + (grouped.getAtomBounds(1).width > grouped.getAtomBounds(0).width * 2));
    }
}
}
