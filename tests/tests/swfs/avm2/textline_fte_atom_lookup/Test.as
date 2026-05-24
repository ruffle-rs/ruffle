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
            new TextElement("ABCDE", new ElementFormat(fd, 18))
        ).createTextLine(null, 400);

        for (var i:int = 0; i <= 5; i++) {
            trace("char " + i + " -> atom " + line.getAtomIndexAtCharIndex(i));
        }
    }
}
}
