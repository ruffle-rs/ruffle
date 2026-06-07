package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBaseline;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var line:TextLine = new TextBlock(
            new TextElement("baseline", new ElementFormat(fd, 24))
        ).createTextLine(null, 400);

        trace("ascent positive: " + (line.ascent > 0));
        trace("descent positive: " + (line.descent > 0));
        trace("roman zero: " + near(line.getBaselinePosition(TextBaseline.ROMAN), 0));
        trace("ascent matches: " + near(line.getBaselinePosition(TextBaseline.ASCENT), -line.ascent));
        trace("descent matches: " + near(line.getBaselinePosition(TextBaseline.DESCENT), line.descent));
        trace("ideo top matches: " + near(line.getBaselinePosition(TextBaseline.IDEOGRAPHIC_TOP), -line.ascent));
        trace("ideo center matches: " + near(line.getBaselinePosition(TextBaseline.IDEOGRAPHIC_CENTER), (line.descent - line.ascent) / 2));
        trace("ideo bottom matches: " + near(line.getBaselinePosition(TextBaseline.IDEOGRAPHIC_BOTTOM), line.descent));
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.01;
    }
}
}
