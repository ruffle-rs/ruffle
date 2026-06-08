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
            new TextElement("recreate", new ElementFormat(fd, 20))
        );
        var line:TextLine = tb.createTextLine(null, 400);
        var fullLength:int = line.rawTextLength;
        var again:TextLine = tb.recreateTextLine(line, null, 60);

        trace("same object: " + (again == line));
        trace("raw length changed: " + (again.rawTextLength < fullLength));
        trace("atom count: " + again.atomCount);
        trace("specified width: " + again.specifiedWidth);
        trace("result: " + tb.textLineCreationResult);
    }
}
}
