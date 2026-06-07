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
            new TextElement("Atom", new ElementFormat(fd, 18))
        ).createTextLine(null, 400);

        trace("raw length: " + line.rawTextLength);
        trace("atom count: " + line.atomCount);
        for (var i:int = 0; i < line.atomCount; i++) {
            trace(i + ": " + line.getAtomTextBlockBeginIndex(i) + "-" + line.getAtomTextBlockEndIndex(i));
        }
    }
}
}
