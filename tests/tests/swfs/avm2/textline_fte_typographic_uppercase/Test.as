package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;
import flash.text.engine.TypographicCase;

public class Test extends Sprite {
    public function Test() {
        var normal:TextLine = lineWithCase(TypographicCase.DEFAULT);
        var upper:TextLine = lineWithCase(TypographicCase.UPPERCASE);

        trace("uppercase widens mixed text: " + (upper.textWidth > normal.textWidth));
        trace("uppercase preserves atom count: " + (upper.atomCount == normal.atomCount));
        trace("uppercase preserves raw length: " + (upper.rawTextLength == normal.rawTextLength));
    }

    private function lineWithCase(value:String):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var ef:ElementFormat = new ElementFormat(fd, 24);
        ef.typographicCase = value;
        return new TextBlock(new TextElement("Multics 1965", ef)).createTextLine(null, 10000);
    }
}
}
