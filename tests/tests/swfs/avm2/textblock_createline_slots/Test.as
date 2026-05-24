package {
import flash.display.Sprite;
import flash.text.engine.*;

[SWF(width="100", height="100")]
public class Test extends Sprite {
    public function Test() {
        var fd:FontDescription = new FontDescription();
        fd.fontName = "_sans";
        var fmt:ElementFormat = new ElementFormat(fd, 14);
        var block:TextBlock = new TextBlock(new TextElement("Hello", fmt));
        var line:TextLine = block.createTextLine(null, 321);

        trace("rawTextLength: " + line.rawTextLength);
        trace("begin: " + line.textBlockBeginIndex);
        trace("specifiedWidth: " + line.specifiedWidth);
        trace("textBlock linked: " + (line.textBlock === block));
    }
}
}
