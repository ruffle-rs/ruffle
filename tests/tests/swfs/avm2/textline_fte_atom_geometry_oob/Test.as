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
            new TextElement("Z", new ElementFormat(fd, 24))
        ).createTextLine(null, 400);
        var bounds:Rectangle = line.getAtomBounds(5);

        trace("bounds empty: " + (bounds.x == 0 && bounds.y == 0 && bounds.width == 0 && bounds.height == 0));
        trace("center empty: " + line.getAtomCenter(5));
    }
}
}
