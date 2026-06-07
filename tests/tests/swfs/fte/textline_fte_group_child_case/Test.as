package {
import flash.display.Sprite;
import flash.text.engine.ContentElement;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.GroupElement;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;
import flash.text.engine.TypographicCase;

public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";

        var normal:ElementFormat = new ElementFormat(fd, 24);
        var upper:ElementFormat = new ElementFormat(fd, 24);
        upper.typographicCase = TypographicCase.UPPERCASE;

        var ungrouped:TextLine = new TextBlock(new TextElement("miiii", normal)).createTextLine(null, 400);
        var parts:Vector.<ContentElement> = new Vector.<ContentElement>();
        parts.push(new TextElement("m", normal));
        parts.push(new TextElement("iiii", upper));
        var grouped:TextLine = new TextBlock(new GroupElement(parts)).createTextLine(null, 400);

        trace("case changes width: " + (grouped.textWidth > ungrouped.textWidth));
        trace("first atom unchanged: " + near(grouped.getAtomBounds(0).width, ungrouped.getAtomBounds(0).width));
        trace("second atom upper wider: " + (grouped.getAtomBounds(1).width > ungrouped.getAtomBounds(1).width));
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.05;
    }
}
}
