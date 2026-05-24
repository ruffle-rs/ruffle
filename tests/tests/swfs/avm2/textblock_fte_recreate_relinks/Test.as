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
            new TextElement("first\u2029second", new ElementFormat(fd, 20))
        );

        var first:TextLine = block.createTextLine(null, 1000);
        var second:TextLine = block.createTextLine(first, 1000);
        trace("initial chain: " + (first.nextLine === second && second.previousLine === first));
        trace("initial second begin: " + second.textBlockBeginIndex);

        second.validity = "invalid";
        var reused:TextLine = block.recreateTextLine(second, null, 1000);
        trace("same object: " + (reused === second));
        trace("valid after recreate: " + second.validity);
        trace("previous cleared: " + (second.previousLine == null));
        trace("first line rebound: " + (block.firstLine === second));
        trace("recreated begin: " + second.textBlockBeginIndex);
    }
}
}
