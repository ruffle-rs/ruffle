package {
import flash.display.Sprite;
import flash.text.engine.ElementFormat;
import flash.text.engine.FontDescription;
import flash.text.engine.TextBlock;
import flash.text.engine.TextElement;
import flash.text.engine.TextLine;

public class Test extends Sprite {
    public function Test() {
        var small:TextLine = lineAt(8);
        var large:TextLine = lineAt(40);
        trace("small ascent typographic: " + near(small.ascent, 5.82));
        trace("small descent typographic: " + near(small.descent, 1.68));
        trace("large ascent typographic: " + near(large.ascent, 29.12));
        trace("large descent typographic: " + near(large.descent, 8.42));
        trace("ascent scales by size: " + near(large.ascent / small.ascent, 5));
    }

    private function lineAt(size:Number):TextLine {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "Liberation Sans";
        return new TextBlock(
            new TextElement("Multics CTSS", new ElementFormat(fd, size))
        ).createTextLine(null, 10000);
    }

    private function near(a:Number, b:Number):Boolean {
        return Math.abs(a - b) < 0.05;
    }
}
}
