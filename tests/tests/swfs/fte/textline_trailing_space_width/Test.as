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
            new TextElement("A   ", new ElementFormat(fd, 24))
        ).createTextLine(null, 400);
        var last:Rectangle = line.getAtomBounds(line.atomCount - 1);
        var fullAdvance:Number = last.x + last.width;

        trace("has trailing atoms: " + (line.atomCount == 4));
        trace("textWidth trims trailing spaces: " + (line.textWidth < fullAdvance));
        trace("textWidth keeps first glyph: " + (line.textWidth > 0));
    }
}
}
