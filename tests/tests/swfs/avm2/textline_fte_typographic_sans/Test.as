package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        var line:TextLine = new TextBlock(
            new TextElement("Multics CTSS", new ElementFormat(fd, 24))
        ).createTextLine(null, 10000);

        trace("ascent is typographic: " + near(line.ascent, 17.47));
        trace("descent is typographic: " + near(line.descent, 5.05));
        trace("textHeight sums typographic metrics: " + near(line.textHeight, line.ascent + line.descent));
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.05;
    }
}
}
