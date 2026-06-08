package {
import flash.display.Sprite;
import flash.geom.Rectangle;
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
            new TextElement("   ", new ElementFormat(fd, 24))
        ).createTextLine(null, 400);
        var last:Rectangle = line.getAtomBounds(line.atomCount - 1);

        trace("atoms remain visible to API: " + (line.atomCount == 3));
        trace("advance still exists: " + (last.x + last.width > 0));
        trace("textWidth is zero: " + (line.textWidth == 0));
    }
}
}
