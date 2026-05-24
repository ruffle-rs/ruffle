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
        var tb:TextBlock = new TextBlock(
            new TextElement("A supercalifragilistic", new ElementFormat(fd, 18))
        );
        var line:TextLine = tb.createTextLine(null, 400);
        var fullLength:int = line.rawTextLength;
        tb.recreateTextLine(line, null, 80);

        trace("same object narrowed: " + (line.rawTextLength < fullLength));
        trace("begin unchanged: " + line.textBlockBeginIndex);
        trace("specified width: " + line.specifiedWidth);
    }
}
}
