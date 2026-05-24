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
        var upper:TextLine = lineFor("WIDE", TypographicCase.DEFAULT);
        var lower:TextLine = lineFor("WIDE", TypographicCase.LOWERCASE);
        var mixed:TextLine = lineFor("Wide", TypographicCase.LOWERCASE);

        trace("lowercase narrows caps: " + (lower.textWidth < upper.textWidth));
        trace("lowercase preserves atoms: " + (lower.atomCount == upper.atomCount));
        trace("same lowercase text matches: " + near(lower.textWidth, mixed.textWidth));
    }

    private function lineFor(text:String, value:String):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var ef:ElementFormat = new ElementFormat(fd, 24);
        ef.typographicCase = value;
        return new TextBlock(new TextElement(text, ef)).createTextLine(null, 10000);
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.01;
    }
}
}
