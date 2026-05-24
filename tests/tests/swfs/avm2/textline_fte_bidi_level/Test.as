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
        var block:TextBlock = new TextBlock(
            new TextElement("abc", new ElementFormat(fd, 20))
        );
        block.bidiLevel = 1;
        var line:TextLine = block.createTextLine(null, 1000);

        trace("atomCount: " + line.atomCount);
        trace("level first: " + line.getAtomBidiLevel(0));
        trace("level middle: " + line.getAtomBidiLevel(1));
        trace("level before: " + line.getAtomBidiLevel(-1));
        trace("level after: " + line.getAtomBidiLevel(line.atomCount));
    }
}
}
