package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="100", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var fmt:ElementFormat = new ElementFormat(fd, 14);
        var block:TextBlock = new TextBlock(new TextElement("Hi", fmt));
        var first:TextLine = block.createTextLine(null, 321);
        var after:TextLine = block.createTextLine(first, 321);

        trace("firstLine === first: " + (block.firstLine === first));
        trace("lastLine === first: " + (block.lastLine === first));
        trace("complete line: " + after);
        trace("result: " + block.textLineCreationResult);
    }
}
}
